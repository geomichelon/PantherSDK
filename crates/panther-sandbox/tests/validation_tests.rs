use anyhow::Result;
use panther_validation::{LLMValidator, ProviderFactory};

#[tokio::test(flavor = "multi_thread")]
async fn validation_runs_when_providers_available() -> Result<()> {
    let mut providers = Vec::new();
    if let Ok(p) = ProviderFactory::openai_from_env() { providers.push(p); }
    if let Ok(p) = ProviderFactory::ollama_from_env() { providers.push(p); }
    if providers.is_empty() {
        eprintln!("No providers configured; skipping validation test.");
        return Ok(());
    }

    let guide_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../panther-validation/guidelines/anvisa.json");
    let validator = LLMValidator::from_path(&guide_path, providers)?;
    let results = validator.validate("What are the side effects of Ibuprofen?").await?;
    assert!(!results.is_empty(), "should have at least one result");
    // Skip assertion if all providers errored (no live backend available)
    let all_error = results.iter().all(|r| r.adherence_score == 0.0 && r.raw_text.starts_with("error:"));
    if all_error {
        eprintln!("Providers unreachable; skipping score assertion.");
        return Ok(());
    }
    assert!(results[0].adherence_score >= 0.5, "top adherence should be reasonable");
    Ok(())
}
