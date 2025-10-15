use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentPlan {
    ValidateSealAnchor {
        guidelines_json: Option<String>,
        anchor: Option<AnchorCfg>,
        timeouts_ms: Option<Timeouts>,
        retries: Option<Retries>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Timeouts {
    pub validate_ms: Option<u64>,
    pub seal_ms: Option<u64>,
    pub anchor_ms: Option<u64>,
    pub status_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Retries {
    pub validate: Option<u32>,
    pub anchor: Option<u32>,
    pub status: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorCfg {
    pub rpc_url: String,
    pub contract_addr: String,
    pub priv_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCfg {
    #[serde(rename = "type")]
    pub ty: String,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInput {
    pub prompt: String,
    pub providers: Vec<ProviderCfg>,
    pub salt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEvent {
    pub ts: i64,
    pub stage: String,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutcome {
    pub results: Option<Vec<panther_validation::ValidationResult>>,
    pub proof: Option<panther_validation::proof::Proof>,
    pub tx_hash: Option<String>,
    pub anchored: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRunResult {
    pub outcome: AgentOutcome,
    pub events: Vec<AgentEvent>,
}

fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

async fn do_validate(
    prompt: &str,
    providers: &[ProviderCfg],
    guidelines_json: &str,
) -> Result<Vec<panther_validation::ValidationResult>> {
    // Prefer async providers if enabled; otherwise fallback to sync
    #[cfg(feature = "validation-async")]
    {
        use panther_domain::ports::LlmProviderAsync as _;
        let mut list: Vec<(String, Arc<dyn panther_domain::ports::LlmProviderAsync>)> = Vec::new();
        for c in providers {
            match c.ty.as_str() {
                #[cfg(feature = "validation-openai-async")]
                "openai" => {
                    if let (Some(api_key), Some(model)) = (c.api_key.clone(), c.model.clone()) {
                        let base = c
                            .base_url
                            .clone()
                            .unwrap_or_else(|| "https://api.openai.com".to_string());
                        let p = panther_providers::openai_async::OpenAiProviderAsync {
                            api_key,
                            model: model.clone(),
                            base_url: base,
                            timeout_secs: 30,
                            retries: 2,
                        };
                        list.push((format!("openai:{}", model), Arc::new(p)));
                    }
                }
                #[cfg(feature = "validation-ollama-async")]
                "ollama" => {
                    if let (Some(base), Some(model)) = (c.base_url.clone(), c.model.clone()) {
                        let p = panther_providers::ollama_async::OllamaProviderAsync {
                            base_url: base,
                            model: model.clone(),
                            timeout_secs: 30,
                            retries: 2,
                        };
                        list.push((format!("ollama:{}", model), Arc::new(p)));
                    }
                }
                _ => {}
            }
        }
        if !list.is_empty() {
            let validator = panther_validation::LLMValidatorAsync::from_json_str(guidelines_json, list)?;
            return validator.validate(prompt).await;
        }
    }

    // Sync fallback
    let list: Vec<(String, Arc<dyn panther_domain::ports::LlmProvider>)> = Vec::new();
    for c in providers {
        match c.ty.as_str() {
            #[cfg(feature = "validation-openai")]
            "openai" => {
                if let (Some(api_key), Some(model)) = (c.api_key.clone(), c.model.clone()) {
                    let base = c
                        .base_url
                        .clone()
                        .unwrap_or_else(|| "https://api.openai.com".to_string());
                    let p = panther_providers::openai::OpenAiProvider {
                        api_key,
                        model: model.clone(),
                        base_url: base,
                    };
                    list.push((format!("openai:{}", model), Arc::new(p)));
                }
            }
            #[cfg(feature = "validation-ollama")]
            "ollama" => {
                if let (Some(base), Some(model)) = (c.base_url.clone(), c.model.clone()) {
                    let p = panther_providers::ollama::OllamaProvider { base_url: base, model: model.clone() };
                    list.push((format!("ollama:{}", model), Arc::new(p)));
                }
            }
            _ => {}
        }
    }
    if list.is_empty() {
        anyhow::bail!("no providers configured")
    }
    let validator = panther_validation::LLMValidator::from_json_str(guidelines_json, list)?;
    validator.validate(prompt).await
}

pub async fn run_plan_async(plan: AgentPlan, input: AgentInput) -> Result<AgentRunResult> {
    let mut events: Vec<AgentEvent> = Vec::new();
    let mut outcome = AgentOutcome { results: None, proof: None, tx_hash: None, anchored: None };

    match plan {
        AgentPlan::ValidateSealAnchor { guidelines_json, anchor, timeouts_ms, retries } => {
            // Resolve guidelines JSON string (default to built-in ANVISA if not provided)
            let guidelines_json = match guidelines_json {
                Some(s) => s,
                None => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../panther-validation/guidelines/anvisa.json")).to_string(),
            };

            // Defaults
            let t_validate = timeouts_ms.as_ref().and_then(|t| t.validate_ms).unwrap_or(30_000);
            let _t_anchor = timeouts_ms.as_ref().and_then(|t| t.anchor_ms).unwrap_or(30_000);
            let _t_status = timeouts_ms.as_ref().and_then(|t| t.status_ms).unwrap_or(5_000);
            let r_validate = retries.as_ref().and_then(|r| r.validate).unwrap_or(0);
            let _r_anchor = retries.as_ref().and_then(|r| r.anchor).unwrap_or(0);
            let _r_status = retries.as_ref().and_then(|r| r.status).unwrap_or(0);

            // ---- Validate (with retries/timeout) ----
            events.push(AgentEvent { ts: now_ms(), stage: "validate".into(), message: format!("starting validation (retries={})", r_validate).into(), data: None });
            let mut _last_err: Option<anyhow::Error> = None;
            let mut attempt = 0u32;
            let results = loop {
                events.push(AgentEvent { ts: now_ms(), stage: "validate".into(), message: format!("attempt {}", attempt + 1), data: None });
                let fut = do_validate(&input.prompt, &input.providers, &guidelines_json);
                match tokio::time::timeout(Duration::from_millis(t_validate), fut).await {
                    Ok(Ok(v)) => break v,
                    Ok(Err(e)) => { _last_err = Some(e); }
                    Err(_) => { _last_err = Some(anyhow::anyhow!("timeout: validate exceeded {} ms", t_validate)); }
                }
                if attempt >= r_validate { break Err(_last_err.unwrap_or_else(|| anyhow::anyhow!("validate failed")))?; }
                // backoff with jitter
                let wait = (200u64.saturating_mul(1 << attempt.min(4))) + ((attempt as u64 * 37) % 120);
                tokio::time::sleep(Duration::from_millis(wait.min(2_000))).await;
                attempt += 1;
            };
            events.push(AgentEvent { ts: now_ms(), stage: "validate".into(), message: "validation complete".into(), data: Some(serde_json::to_value(&results).unwrap_or(Value::Null)) });
            outcome.results = Some(results.clone());

            events.push(AgentEvent { ts: now_ms(), stage: "seal".into(), message: "computing proof".into(), data: None });
            let providers_json = serde_json::to_string(&input.providers).unwrap_or_else(|_| "[]".to_string());
            let results_json = serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string());
            let ctx = panther_validation::proof::ProofContext { sdk_version: env!("CARGO_PKG_VERSION").to_string(), salt: input.salt.clone() };
            let proof = panther_validation::proof::compute_proof(&input.prompt, &providers_json, &guidelines_json, &results_json, &ctx)?;
            events.push(AgentEvent { ts: now_ms(), stage: "seal".into(), message: "proof computed".into(), data: Some(serde_json::to_value(&proof).unwrap_or(Value::Null)) });
            outcome.proof = Some(proof.clone());

            if let Some(a) = anchor {
                #[cfg(feature = "blockchain-eth")]
                {
                    let t_anchor = _t_anchor; let r_anchor = _r_anchor; let t_status = _t_status; let r_status = _r_status;
                    events.push(AgentEvent { ts: now_ms(), stage: "anchor".into(), message: format!("anchoring on-chain (retries={})", r_anchor), data: None });
                    let mut last_err: Option<anyhow::Error> = None;
                    let mut attempt = 0u32;
                    let res = loop {
                        events.push(AgentEvent { ts: now_ms(), stage: "anchor".into(), message: format!("attempt {}", attempt + 1), data: None });
                        let fut = panther_validation::anchor_eth::anchor_proof(&proof.combined_hash, &a.rpc_url, &a.contract_addr, &a.priv_key);
                        match tokio::time::timeout(Duration::from_millis(t_anchor), fut).await {
                            Ok(Ok(v)) => break v,
                            Ok(Err(e)) => { last_err = Some(e); }
                            Err(_) => { last_err = Some(anyhow::anyhow!("timeout: anchor exceeded {} ms", t_anchor)); }
                        }
                        if attempt >= r_anchor { break Err(last_err.unwrap_or_else(|| anyhow::anyhow!("anchor failed")))?; }
                        let wait = (300u64.saturating_mul(1 << attempt.min(4))) + ((attempt as u64 * 41) % 150);
                        tokio::time::sleep(Duration::from_millis(wait.min(3_000))).await;
                        attempt += 1;
                    };
                    events.push(AgentEvent { ts: now_ms(), stage: "anchor".into(), message: "anchor tx submitted".into(), data: Some(serde_json::to_value(&res).ok()) });
                    outcome.tx_hash = Some(res.tx_hash.clone());
                    // Optional status check with retries/timeout
                    let mut last_err: Option<anyhow::Error> = None;
                    let mut attempt = 0u32;
                    let anchored = loop {
                        events.push(AgentEvent { ts: now_ms(), stage: "status".into(), message: format!("checking status (attempt {} of {})", attempt + 1, r_status + 1), data: None });
                        let fut = panther_validation::anchor_eth::is_anchored(&proof.combined_hash, &a.rpc_url, &a.contract_addr);
                        match tokio::time::timeout(Duration::from_millis(t_status), fut).await {
                            Ok(Ok(v)) => break v,
                            Ok(Err(e)) => { last_err = Some(e); }
                            Err(_) => { last_err = Some(anyhow::anyhow!("timeout: status exceeded {} ms", t_status)); }
                        }
                        if attempt >= r_status { break false; }
                        let wait = (500u64.saturating_mul(1 << attempt.min(3))) + ((attempt as u64 * 47) % 200);
                        tokio::time::sleep(Duration::from_millis(wait.min(4_000))).await;
                        attempt += 1;
                    };
                    outcome.anchored = Some(anchored);
                    events.push(AgentEvent { ts: now_ms(), stage: "status".into(), message: "status checked".into(), data: Some(serde_json::json!({"anchored": anchored})) });
                }
                #[cfg(not(feature = "blockchain-eth"))]
                {
                    let _ = &a; // silence unused when feature is off
                    events.push(AgentEvent { ts: now_ms(), stage: "anchor".into(), message: "blockchain feature disabled".into(), data: None });
                }
            }
        }
    }

    Ok(AgentRunResult { outcome, events })
}

pub fn run_plan(plan_json: &str, input_json: &str) -> Result<AgentRunResult> {
    let plan: AgentPlan = serde_json::from_str(plan_json)?;
    let input: AgentInput = serde_json::from_str(input_json)?;
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    rt.block_on(run_plan_async(plan, input))
}
