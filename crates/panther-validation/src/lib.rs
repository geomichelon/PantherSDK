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
