use panther_domain::entities::{Completion, Prompt};
use panther_domain::ports::LlmProvider;

pub struct NullProvider;

impl LlmProvider for NullProvider {
    fn generate(&self, prompt: &Prompt) -> anyhow::Result<Completion> {
        Ok(Completion { text: format!("echo: {}", prompt.text), model: Some(self.name().into()) })
    }
    fn name(&self) -> &'static str { "null" }
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
