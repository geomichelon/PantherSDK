use anyhow::Result;
use panther_validation::{LLMValidator, ProviderFactory, ValidationResult};
use std::fs;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let mut providers = Vec::new();
    if let Ok(p) = ProviderFactory::openai_from_env() { providers.push(p); }
    if let Ok(p) = ProviderFactory::ollama_from_env() { providers.push(p); }
    if providers.is_empty() {
        eprintln!("No providers configured. Set OpenAI env or run Ollama.");
        std::process::exit(2);
    }

    let guides = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../panther-validation/guidelines/anvisa.json");
    let validator = LLMValidator::from_path(&guides, providers)?;

    let prompt = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let prompt = if prompt.is_empty() { "Explain insulin function".to_string() } else { prompt };

    let results = validator.validate(&prompt).await?;
    print_table(&results);

    fs::create_dir_all("outputs").ok();
    fs::write("outputs/validation_results.json", serde_json::to_string_pretty(&results)?)?;
    Ok(())
}

fn print_table(results: &[ValidationResult]) {
    println!("\n🧩 LLM Validation Summary\n────────────────────────────");
    for r in results {
        println!(
            "{:<18} → {:>5.1}% | {} ms",
            r.provider_name,
            r.adherence_score,
            r.latency_ms
        );
    }
}

