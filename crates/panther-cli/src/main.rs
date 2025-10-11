use anyhow::Result;
use clap::{Parser, Subcommand};
use panther_validation::{LLMValidator, ProviderFactory};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "panther", version, author, about = "PantherSDK CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Validate LLM providers against guidelines
    Validate {
        /// Prompt to validate
        prompt: String,
        /// Path to guidelines JSON (defaults to ANVISA example)
        #[arg(short, long)]
        guidelines: Option<PathBuf>,
    },
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Validate { prompt, guidelines } => {
            let mut providers = Vec::new();
            if let Ok(p) = ProviderFactory::openai_from_env() { providers.push(p); }
            if let Ok(p) = ProviderFactory::ollama_from_env() { providers.push(p); }
            if providers.is_empty() {
                eprintln!("No providers configured. Set PANTHER_OPENAI_API_KEY or run Ollama.");
                std::process::exit(2);
            }

            let default_guides = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../panther-validation/guidelines/anvisa.json");
            let guide_path = guidelines.unwrap_or(default_guides);
            let validator = LLMValidator::from_path(&guide_path, providers)?;
            let results = validator.validate(&prompt).await?;

            println!("\n🧩 LLM Validation Summary\n────────────────────────────");
            for r in &results {
                println!(
                    "{:<18} → {:>5.1}% | {} ms",
                    r.provider_name,
                    r.adherence_score,
                    r.latency_ms
                );
            }
        }
    }
    Ok(())
}

