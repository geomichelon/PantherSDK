use anyhow::Result;
use panther_domain::entities::Prompt;
use panther_domain::ports::LlmProvider;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::task;

// Explicit result alias to help type inference in tooling (rust-analyzer)
type VRes = anyhow::Result<ValidationResult>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guideline {
    pub topic: String,
    pub expected_terms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub provider_name: String,
    pub adherence_score: f64,
    pub missing_terms: Vec<String>,
    pub latency_ms: i64,
    pub cost: Option<f64>,
    pub raw_text: String,
}

pub struct LLMValidator {
    guidelines: Vec<Guideline>,
    providers: Vec<(String, Arc<dyn LlmProvider>)>,
}

impl LLMValidator {
    pub fn from_path<P: AsRef<Path>>(path: P, providers: Vec<(String, Arc<dyn LlmProvider>)>) -> Result<Self> {
        let text = fs::read_to_string(path)?;
        let guidelines: Vec<Guideline> = serde_json::from_str(&text)?;
        Ok(Self { guidelines, providers })
    }

    pub fn from_json_str(json: &str, providers: Vec<(String, Arc<dyn LlmProvider>)>) -> Result<Self> {
        let guidelines: Vec<Guideline> = serde_json::from_str(json)?;
        Ok(Self { guidelines, providers })
    }

    pub async fn validate(&self, input_prompt: &str) -> Result<Vec<ValidationResult>> {
        let expected: Vec<String> = self
            .guidelines
            .iter()
            .flat_map(|g| g.expected_terms.iter().cloned())
            .collect();

        let mut tasks: Vec<tokio::task::JoinHandle<VRes>> = Vec::new();
        for (label, prov) in &self.providers {
            let label = label.clone();
            let prov = prov.clone();
            let prompt = Prompt { text: input_prompt.to_string() };
            let expected_terms = expected.clone();
            tasks.push(task::spawn_blocking(move || -> VRes {
                let start = now_ms();
                let res = prov.generate(&prompt);
                let end = now_ms();
                match res {
                    Ok(c) => {
                        let (score, missing) = score_text(&c.text, &expected_terms);
                        Ok::<ValidationResult, anyhow::Error>(ValidationResult {
                            provider_name: label,
                            adherence_score: score,
                            missing_terms: missing,
                            latency_ms: end - start,
                            cost: None,
                            raw_text: c.text,
                        })
                    }
                    Err(e) => Ok::<ValidationResult, anyhow::Error>(ValidationResult {
                        provider_name: label,
                        adherence_score: 0.0,
                        missing_terms: expected_terms,
                        latency_ms: end - start,
                        cost: None,
                        raw_text: format!("error: {}", e),
                    }),
                }
            }));
        }

        let mut results = Vec::new();
        for t in tasks {
            match t.await {
                Ok(Ok(vr)) => results.push(vr),
                Ok(Err(e)) => eprintln!("task error: {}", e),
                Err(e) => eprintln!("join error: {}", e),
            }
        }
        results.sort_by(|a, b| b.adherence_score.partial_cmp(&a.adherence_score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(results)
    }
}

fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as i64
}

fn score_text(text: &str, expected_terms: &[String]) -> (f64, Vec<String>) {
    if expected_terms.is_empty() { return (100.0, vec![]); }
    let lower = text.to_lowercase();
    let mut missing = Vec::new();
    for term in expected_terms {
        if !lower.contains(&term.to_lowercase()) { missing.push(term.clone()); }
    }
    let total = expected_terms.len() as f64;
    let penalty = (missing.len() as f64) * (100.0 / total);
    let score = (100.0 - penalty).clamp(0.0, 100.0);
    (score, missing)
}

pub struct ProviderFactory;

impl ProviderFactory {
    pub fn openai_from_env() -> Result<(String, Arc<dyn LlmProvider>)> {
        #[cfg(feature = "openai")]
        {
            let api_key = std::env::var("PANTHER_OPENAI_API_KEY")?;
            let model = std::env::var("PANTHER_OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
            let base = std::env::var("PANTHER_OPENAI_BASE").unwrap_or_else(|_| "https://api.openai.com".to_string());
            let p = panther_providers::openai::OpenAiProvider { api_key, model: model.clone(), base_url: base };
            Ok((format!("openai:{}", model), Arc::new(p)))
        }
        #[cfg(not(feature = "openai"))]
        {
            Err(anyhow::anyhow!("openai feature not enabled"))
        }
    }

    pub fn ollama_from_env() -> Result<(String, Arc<dyn LlmProvider>)> {
        #[cfg(feature = "ollama")]
        {
            let base = std::env::var("PANTHER_OLLAMA_BASE").unwrap_or_else(|_| "http://localhost:11434".to_string());
            let model = std::env::var("PANTHER_OLLAMA_MODEL").unwrap_or_else(|_| "llama3".to_string());
            let p = panther_providers::ollama::OllamaProvider { base_url: base, model: model.clone() };
            Ok((format!("ollama:{}", model), Arc::new(p)))
        }
        #[cfg(not(feature = "ollama"))]
        {
            Err(anyhow::anyhow!("ollama feature not enabled"))
        }
    }
}

// ---- Proofs (Stage 1: offline) ----
pub mod proof {
    use super::*;
    use sha3::{Digest, Sha3_512};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ProofContext {
        pub sdk_version: String,
        pub salt: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Proof {
        pub scheme: String,          // e.g., "panther-proof-v1"
        pub input_hash: String,      // hex(sha3_512(canonical(prompt, providers, guidelines, salt?)))
        pub results_hash: String,    // hex(sha3_512(canonical(results)))
        pub combined_hash: String,   // hex(sha3_512(input_hash || results_hash))
        pub guidelines_hash: String, // hex(sha3_512(canonical(guidelines)))
        pub providers_hash: String,  // hex(sha3_512(canonical(providers)))
        pub timestamp_ms: i64,
        pub sdk_version: String,
        pub salt_present: bool,
    }

    fn canonicalize(value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => {
                let mut entries: Vec<_> = map.iter().collect();
                entries.sort_by(|a, b| a.0.cmp(b.0));
                let mut new = serde_json::Map::new();
                for (k, v) in entries {
                    new.insert(k.clone(), canonicalize(v));
                }
                serde_json::Value::Object(new)
            }
            serde_json::Value::Array(arr) => {
                let v = arr.iter().map(canonicalize).collect::<Vec<_>>();
                serde_json::Value::Array(v)
            }
            _ => value.clone(),
        }
    }

    fn hash_json(value: &serde_json::Value) -> String {
        let canon = canonicalize(value);
        let bytes = serde_json::to_vec(&canon).unwrap_or_default();
        let mut hasher = Sha3_512::new();
        hasher.update(&bytes);
        let out = hasher.finalize();
        hex::encode(out)
    }

    fn hash_concat_hex(a_hex: &str, b_hex: &str) -> String {
        let mut hasher = Sha3_512::new();
        hasher.update(a_hex.as_bytes());
        hasher.update(b_hex.as_bytes());
        let out = hasher.finalize();
        hex::encode(out)
    }

    pub fn compute_proof(
        prompt: &str,
        providers_json: &str,
        guidelines_json: &str,
        results_json: &str,
        ctx: &ProofContext,
    ) -> anyhow::Result<Proof> {
        let providers_val: serde_json::Value = serde_json::from_str(providers_json).unwrap_or(serde_json::Value::Null);
        let guidelines_val: serde_json::Value = serde_json::from_str(guidelines_json).unwrap_or(serde_json::Value::Null);
        let results_val: serde_json::Value = serde_json::from_str(results_json).unwrap_or(serde_json::Value::Null);

        let providers_hash = hash_json(&providers_val);
        let guidelines_hash = hash_json(&guidelines_val);
        let results_hash = hash_json(&results_val);

        // input bundle: prompt + providers + guidelines + optional salt
        let input_bundle = serde_json::json!({
            "prompt": prompt,
            "providers": providers_val,
            "guidelines": guidelines_val,
            "salt": ctx.salt,
        });
        let input_hash = hash_json(&input_bundle);
        let combined_hash = hash_concat_hex(&input_hash, &results_hash);
        let proof = Proof {
            scheme: "panther-proof-v1".to_string(),
            input_hash,
            results_hash,
            combined_hash,
            guidelines_hash,
            providers_hash,
            timestamp_ms: now_ms(),
            sdk_version: ctx.sdk_version.clone(),
            salt_present: ctx.salt.is_some(),
        };
        Ok(proof)
    }

    pub fn verify_proof_local(
        expected: &Proof,
        prompt: &str,
        providers_json: &str,
        guidelines_json: &str,
        results_json: &str,
        salt: Option<String>,
    ) -> bool {
        let ctx = ProofContext { sdk_version: expected.sdk_version.clone(), salt };
        if let Ok(p) = compute_proof(prompt, providers_json, guidelines_json, results_json, &ctx) {
            p.combined_hash == expected.combined_hash
        } else {
            false
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn proof_determinism_and_verify() {
            let prompt = "hello";
            let providers = "[{\"type\":\"ollama\",\"base_url\":\"http://localhost:11434\",\"model\":\"llama3\"}]";
            let guidelines = "[]";
            let results = "[]";
            let ctx = ProofContext { sdk_version: "test".into(), salt: Some("s1".into()) };

            let p1 = compute_proof(prompt, providers, guidelines, results, &ctx).unwrap();
            let p2 = compute_proof(prompt, providers, guidelines, results, &ctx).unwrap();
            assert_eq!(p1.combined_hash, p2.combined_hash);

            assert!(verify_proof_local(&p1, prompt, providers, guidelines, results, Some("s1".into())));
            assert!(!verify_proof_local(&p1, prompt, providers, guidelines, results, Some("different".into())));
        }
    }
}

// ---- On-chain anchoring (Stage 2: Ethereum/Polygon testnet) ----
#[cfg(feature = "blockchain-eth")]
pub mod anchor_eth {
    use super::*;
    use ethers::abi::Abi;
    use ethers::contract::Contract;
    use ethers::middleware::SignerMiddleware;
    use ethers::prelude::k256::ecdsa::SigningKey;
    use ethers::prelude::Wallet;
    use ethers::providers::{Http, Provider};
    use ethers::signers::{LocalWallet, Signer};
    use ethers::types::{Address, H256};
    use std::str::FromStr;
    use std::sync::Arc;
    use std::time::Duration;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AnchorResult { pub tx_hash: String }

    const ABI_JSON: &str = r#"[
      {"inputs":[{"internalType":"bytes32","name":"h","type":"bytes32"}],"name":"anchor","outputs":[],"stateMutability":"nonpayable","type":"function"},
      {"inputs":[{"internalType":"bytes32","name":"h","type":"bytes32"}],"name":"isAnchored","outputs":[{"internalType":"bool","name":"","type":"bool"}],"stateMutability":"view","type":"function"}
    ]"#;

    fn h256_from_hex(s: &str) -> anyhow::Result<H256> {
        let s = s.trim();
        let s = s.strip_prefix("0x").unwrap_or(s);
        let mut bytes = hex::decode(s)?;
        if bytes.len() > 32 { anyhow::bail!("hash too long"); }
        if bytes.len() < 32 {
            let mut padded = vec![0u8; 32 - bytes.len()];
            padded.extend(bytes);
            bytes = padded;
        }
        Ok(H256::from_slice(&bytes))
    }

    pub async fn anchor_proof(
        proof_hash_hex: &str,
        rpc_url: &str,
        contract_addr: &str,
        priv_key_hex: &str,
    ) -> anyhow::Result<AnchorResult> {
        let provider = Provider::<Http>::try_from(rpc_url)?.interval(Duration::from_millis(1000));
        let chain_id = provider.get_chainid().await?.as_u64();
        let wallet: LocalWallet = LocalWallet::from_str(priv_key_hex)?.with_chain_id(chain_id);
        let client = Arc::new(SignerMiddleware::new(provider, wallet));

        let addr: Address = Address::from_str(contract_addr)?;
        let abi: Abi = serde_json::from_str(ABI_JSON)?;
        let contract = Contract::new(addr, abi, client);
        let h = h256_from_hex(proof_hash_hex)?;

        // anchor(bytes32)
        let pending = contract.method::<_, ()>("anchor", h)?.send().await?;
        let tx_hash = format!("{:#x}", pending.tx_hash());
        // optional wait: let _receipt = pending.await?;
        Ok(AnchorResult { tx_hash })
    }

    pub async fn is_anchored(
        proof_hash_hex: &str,
        rpc_url: &str,
        contract_addr: &str,
    ) -> anyhow::Result<bool> {
        let provider = Provider::<Http>::try_from(rpc_url)?;
        let addr: Address = Address::from_str(contract_addr)?;
        let abi: Abi = serde_json::from_str(ABI_JSON)?;
        let contract = Contract::new(addr, abi, Arc::new(provider));
        let h = h256_from_hex(proof_hash_hex)?;
        let anchored: bool = contract.method::<_, bool>("isAnchored", h)?.call().await?;
        Ok(anchored)
    }
}
