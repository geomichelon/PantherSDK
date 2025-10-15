pub mod entities {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ModelSpec {
        pub name: String,
        pub max_tokens: Option<u32>,
        pub temperature: Option<f32>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Prompt {
        pub text: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Completion {
        pub text: String,
        pub model: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TraceEvent {
        pub name: String,
        pub message: String,
        pub timestamp_ms: i64,
        pub attributes: serde_json::Value,
    }
}

pub mod ports {
    use crate::entities::{Completion, Prompt, TraceEvent};
    use async_trait::async_trait;

    pub trait LlmProvider: Send + Sync {
        fn generate(&self, prompt: &Prompt) -> anyhow::Result<Completion>;
        fn name(&self) -> &'static str { "unknown" }
    }

    pub trait TelemetrySink: Send + Sync {
        fn record(&self, event: TraceEvent);
    }

    #[async_trait]
    pub trait LlmProviderAsync: Send + Sync {
        async fn generate(&self, prompt: &Prompt) -> anyhow::Result<Completion>;
        fn name(&self) -> &'static str { "unknown" }
    }

    pub trait MetricsSink: Send + Sync {
        fn inc_counter(&self, name: &str, value: f64);
        fn observe_histogram(&self, name: &str, value: f64);
    }

    pub trait KeyValueStore: Send + Sync {
        fn get(&self, key: &str) -> anyhow::Result<Option<String>>;
        fn set(&self, key: &str, value: String) -> anyhow::Result<()>;
        fn delete(&self, key: &str) -> anyhow::Result<()>;
    }

    // Content metrics port for hexagonal architecture
    pub trait ContentMetrics: Send + Sync {
        fn accuracy(&self, expected: &str, generated: &str) -> f64;
        fn bleu(&self, reference: &str, candidate: &str) -> f64;
        fn coherence(&self, text: &str) -> f64;
        fn diversity(&self, samples: &[String]) -> f64;
        fn fluency(&self, text: &str) -> f64;
        fn rouge_l(&self, reference: &str, candidate: &str) -> f64;
        fn fact_coverage(&self, facts: &[String], candidate: &str) -> f64;
        fn factcheck_adv(&self, facts: &[String], candidate: &str) -> f64;
        fn plagiarism(&self, corpus: &[String], candidate: &str) -> f64;
        fn plagiarism_ngram(&self, corpus: &[String], candidate: &str, n: usize) -> f64;
    }
}

#[cfg(test)]
mod tests {
    use super::entities::*;
    #[test]
    fn prompt_serializes() {
        let p = Prompt { text: "test".into() };
        let s = serde_json::to_string(&p).unwrap();
        assert!(s.contains("test"));
    }
}

pub mod errors {
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum PantherError {
        #[error("provider error: {0}")]
        Provider(String),
        #[error("invalid input: {0}")]
        InvalidInput(String),
    }
}
