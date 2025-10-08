use panther_domain::entities::{Completion, Prompt};
use panther_domain::ports::LlmProvider;

pub struct NullProvider;

impl LlmProvider for NullProvider {
    fn generate(&self, prompt: &Prompt) -> anyhow::Result<Completion> {
        Ok(Completion { text: format!("echo: {}", prompt.text), model: Some(self.name().into()) })
    }
    fn name(&self) -> &'static str { "null" }
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
