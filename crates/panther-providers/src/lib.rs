use panther_domain::entities::{Completion, Prompt};
use panther_domain::ports::LlmProvider;
#[cfg(any(feature = "openai-async", feature = "ollama-async"))]
use panther_domain::ports::LlmProviderAsync;

pub struct NullProvider;

impl LlmProvider for NullProvider {
    fn generate(&self, prompt: &Prompt) -> anyhow::Result<Completion> {
        Ok(Completion { text: format!("echo: {}", prompt.text), model: Some(self.name().into()) })
    }
    fn name(&self) -> &'static str { "null" }
}

#[cfg(feature = "openai-async")]
pub mod openai_async {
    use super::*;
    use async_trait::async_trait;
    use reqwest::StatusCode;

    #[derive(Clone)]
    pub struct OpenAiProviderAsync {
        pub api_key: String,
        pub model: String,
        pub base_url: String,
        pub timeout_secs: u64,
        pub retries: u32,
    }

    #[async_trait]
    impl LlmProviderAsync for OpenAiProviderAsync {
        async fn generate(&self, prompt: &Prompt) -> anyhow::Result<Completion> {
            let url = format!("{}/v1/chat/completions", self.base_url.trim_end_matches('/'));
            let body = serde_json::json!({
                "model": self.model,
                "messages": [
                    {"role": "user", "content": prompt.text}
                ],
                "temperature": 0.2
            });
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(self.timeout_secs.max(1)))
                .build()?;
            let mut last_err: Option<anyhow::Error> = None;
            let mut delay_ms: u64 = 200;
            for attempt in 0..=self.retries {
                let res = client
                    .post(&url)
                    .bearer_auth(&self.api_key)
                    .json(&body)
                    .send()
                    .await;
                match res {
                    Ok(resp) => {
                        let status = resp.status();
                        let v: serde_json::Value = resp.json().await.unwrap_or(serde_json::json!({"error":"invalid json"}));
                        if status.is_success() {
                            let text = v["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
                            return Ok(Completion { text, model: Some(self.model.clone()) });
                        }
                        // categorize errors
                        let category = match status {
                            StatusCode::TOO_MANY_REQUESTS => "rate_limit",
                            StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => "invalid_request",
                            _ if status.is_server_error() => "upstream_error",
                            _ => "request_error",
                        };
                        last_err = Some(anyhow::anyhow!("{}: openai [{}]: {}", category, status, v));
                    }
                    Err(e) => {
                        let category = if e.is_timeout() { "timeout" } else { "network_error" };
                        last_err = Some(anyhow::anyhow!("{}: openai request failed: {}", category, e));
                    }
                }
                // backoff exponencial com limite
                if attempt < self.retries {
                    // deterministic jitter: spread 0..99ms based on attempt
                    let jitter = ((attempt as u64 * 37) % 100) as u64;
                    let wait = (delay_ms + jitter).min(2000);
                    tokio::time::sleep(std::time::Duration::from_millis(wait)).await;
                    delay_ms = delay_ms.saturating_mul(2);
                }
            }
            Err(last_err.unwrap_or_else(|| anyhow::anyhow!("openai unknown error")))
        }
        fn name(&self) -> &'static str { "openai" }
    }
}

#[cfg(feature = "ollama-async")]
pub mod ollama_async {
    use super::*;
    use async_trait::async_trait;
    use reqwest::StatusCode;

    #[derive(Clone)]
    pub struct OllamaProviderAsync {
        pub base_url: String,
        pub model: String,
        pub timeout_secs: u64,
        pub retries: u32,
    }

    #[async_trait]
    impl LlmProviderAsync for OllamaProviderAsync {
        async fn generate(&self, prompt: &Prompt) -> anyhow::Result<Completion> {
            let url = format!("{}/api/generate", self.base_url.trim_end_matches('/'));
            let body = serde_json::json!({
                "model": self.model,
                "prompt": prompt.text,
                "stream": false
            });
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(self.timeout_secs.max(1)))
                .build()?;
            let mut last_err: Option<anyhow::Error> = None;
            let mut delay_ms: u64 = 200;
            for attempt in 0..=self.retries {
                let res = client.post(&url).json(&body).send().await;
                match res {
                    Ok(resp) => {
                        let status = resp.status();
                        let v: serde_json::Value = resp.json().await.unwrap_or(serde_json::json!({"error":"invalid json"}));
                        if status.is_success() {
                            let text = v["response"].as_str().unwrap_or("").to_string();
                            return Ok(Completion { text, model: Some(self.model.clone()) });
                        }
                        let category = match status {
                            StatusCode::TOO_MANY_REQUESTS => "rate_limit",
                            StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => "invalid_request",
                            _ if status.is_server_error() => "upstream_error",
                            _ => "request_error",
                        };
                        last_err = Some(anyhow::anyhow!("{}: ollama [{}]: {}", category, status, v));
                    }
                    Err(e) => {
                        let category = if e.is_timeout() { "timeout" } else { "network_error" };
                        last_err = Some(anyhow::anyhow!("{}: ollama request failed: {}", category, e));
                    }
                }
                if attempt < self.retries {
                    let jitter = ((attempt as u64 * 41) % 120) as u64;
                    let wait = (delay_ms + jitter).min(2000);
                    tokio::time::sleep(std::time::Duration::from_millis(wait)).await;
                    delay_ms = delay_ms.saturating_mul(2);
                }
            }
            Err(last_err.unwrap_or_else(|| anyhow::anyhow!("ollama unknown error")))
        }
        fn name(&self) -> &'static str { "ollama" }
    }
}

#[cfg(feature = "openai")]
pub mod openai {
    use super::*;

    #[derive(Clone)]
    pub struct OpenAiProvider {
        pub api_key: String,
        pub model: String,
        pub base_url: String,
    }

    impl LlmProvider for OpenAiProvider {
        fn generate(&self, prompt: &Prompt) -> anyhow::Result<Completion> {
            let url = format!("{}/v1/chat/completions", self.base_url.trim_end_matches('/'));
            let body = serde_json::json!({
                "model": self.model,
                "messages": [
                    {"role": "user", "content": prompt.text}
                ],
                "temperature": 0.2
            });
            let client = reqwest::blocking::Client::new();
            let res = client
                .post(url)
                .bearer_auth(&self.api_key)
                .json(&body)
                .send()?;
            let status = res.status();
            let v: serde_json::Value = res.json()?;
            if !status.is_success() {
                return Err(anyhow::anyhow!("openai error: {}", v));
            }
            let text = v["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
            Ok(Completion { text, model: Some(self.model.clone()) })
        }
        fn name(&self) -> &'static str { "openai" }
    }
}

#[cfg(feature = "ollama")]
pub mod ollama {
    use super::*;

    #[derive(Clone)]
    pub struct OllamaProvider {
        pub base_url: String,
        pub model: String,
    }

    impl LlmProvider for OllamaProvider {
        fn generate(&self, prompt: &Prompt) -> anyhow::Result<Completion> {
            let url = format!("{}/api/generate", self.base_url.trim_end_matches('/'));
            let body = serde_json::json!({
                "model": self.model,
                "prompt": prompt.text,
                "stream": false
            });
            let client = reqwest::blocking::Client::new();
            let res = client.post(url).json(&body).send()?;
            let status = res.status();
            let v: serde_json::Value = res.json()?;
            if !status.is_success() {
                return Err(anyhow::anyhow!("ollama error: {}", v));
            }
            let text = v["response"].as_str().unwrap_or("").to_string();
            Ok(Completion { text, model: Some(self.model.clone()) })
        }
        fn name(&self) -> &'static str { "ollama" }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use panther_domain::ports::LlmProvider;

    #[test]
    fn null_provider_echoes() {
        let p = NullProvider;
        let out = p.generate(&Prompt { text: "hi".into() }).unwrap();
        assert_eq!(out.text, "echo: hi");
        assert_eq!(out.model.as_deref(), Some("null"));
    }
}
