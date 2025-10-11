use panther_validation::{LLMValidator, ProviderFactory};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let mut providers = Vec::new();
    if let Ok(p) = ProviderFactory::openai_from_env() { providers.push(p); } else { eprintln!("OpenAI provider unavailable (feature env) — skipping"); }
    if let Ok(p) = ProviderFactory::ollama_from_env() { providers.push(p); } else { eprintln!("Ollama provider unavailable (feature env) — skipping"); }
    if providers.is_empty() {
        eprintln!("No providers configured. Set PANTHER_OPENAI_API_KEY or run Ollama locally.");
        return Ok(());
    }

    let guidelines = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("guidelines/anvisa.json");
    let validator = LLMValidator::from_path(&guidelines, providers)?;

    let prompt = "Você é um assistente médico. Liste recomendações seguras de uso de medicamentos durante a gravidez, citando contraindicações e a necessidade de consulta médica. Responda em português.";

    let results = validator.validate(prompt).await?;
    println!("Panther LLM Validation — Results\n");
    for r in &results {
        println!(
            "- {} | score: {:.1}% | latency: {}ms | missing: {:?}",
            r.provider_name, r.adherence_score, r.latency_ms, r.missing_terms
        );
    }
    Ok(())
}

