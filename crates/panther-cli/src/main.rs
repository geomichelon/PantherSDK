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
    /// Proof operations (Stage 1/2/3)
    Proof {
        #[command(subcommand)]
        cmd: ProofCmd,
    },
}

#[derive(Subcommand, Debug)]
enum ProofCmd {
    /// Check on-chain status via backend API
    Status {
        /// Combined hash (0x… or raw hex)
        hash: String,
        /// API base (default http://127.0.0.1:8000)
        #[arg(long)]
        api_base: Option<String>,
        /// API key for the backend (optional)
        #[arg(long)]
        api_key: Option<String>,
    },
    /// List proof history via backend API
    History {
        /// Optional filter hash (0x…)
        #[arg(long)]
        hash: Option<String>,
        /// API base (default http://127.0.0.1:8000)
        #[arg(long)]
        api_base: Option<String>,
        /// Limit (default 50)
        #[arg(long, default_value = "50")]
        limit: usize,
        /// API key (optional)
        #[arg(long)]
        api_key: Option<String>,
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
            // Summary p50/p95 and error counts
            if !results.is_empty() {
                let mut lats: Vec<i64> = results.iter().map(|r| r.latency_ms).collect();
                lats.sort();
                let idx = |p: f64| -> usize {
                    if lats.is_empty() { 0 } else { ((p * (lats.len() as f64 - 1.0)).round() as isize).clamp(0, lats.len() as isize - 1) as usize }
                };
                let p50 = lats[idx(0.50)];
                let p95 = lats[idx(0.95)];
                let mut err = 0usize;
                for r in &results { if r.adherence_score == 0.0 && r.raw_text.starts_with("error") { err += 1; } }
                println!("\nSummary: p50={} ms, p95={} ms, errors={}/{}", p50, p95, err, results.len());

                // Per-provider error counts
                use std::collections::HashMap;
                let mut per: HashMap<String, (usize, usize, Vec<i64>)> = HashMap::new();
                for r in &results {
                    let e = per.entry(r.provider_name.clone()).or_insert((0,0,Vec::new()));
                    e.0 += 1; // total
                    if r.adherence_score == 0.0 && (r.raw_text.starts_with("error") || r.raw_text.starts_with("{")) { e.1 += 1; }
                    e.2.push(r.latency_ms);
                }
                println!("\nBy provider:");
                for (prov, (tot, errs, mut latv)) in per {
                    latv.sort();
                    let mid = latv[latv.len()/2];
                    println!("  - {:<18} total={} errors={} p50={} ms", prov, tot, errs, mid);
                }
            }
        }
        Commands::Proof { cmd } => {
            match cmd {
                ProofCmd::Status { hash, api_base, api_key } => {
                    let base = api_base.unwrap_or_else(|| "http://127.0.0.1:8000".to_string());
                    let h = if hash.starts_with("0x") { hash } else { format!("0x{}", hash) };
                    let url = format!("{}/proof/status?hash={}", base.trim_end_matches('/'), h);
                    let client = reqwest::blocking::Client::new();
                    let mut req = client.get(&url);
                    if let Some(k) = api_key { req = req.header("X-API-Key", k); }
                    let resp = req.send()?;
                    if !resp.status().is_success() {
                        eprintln!("HTTP {}", resp.status());
                        return Ok(());
                    }
                    let v: serde_json::Value = resp.json()?;
                    println!("{}", serde_json::to_string_pretty(&v)?);
                }
                ProofCmd::History { hash, api_base, limit, api_key } => {
                    let base = api_base.unwrap_or_else(|| "http://127.0.0.1:8000".to_string());
                    let mut url = format!("{}/proof/history?limit={}", base.trim_end_matches('/'), limit);
                    if let Some(h) = hash { url.push_str(&format!("&hash={}", h)); }
                    let client = reqwest::blocking::Client::new();
                    let mut req = client.get(&url);
                    if let Some(k) = api_key { req = req.header("X-API-Key", k); }
                    let resp = req.send()?;
                    if !resp.status().is_success() {
                        eprintln!("HTTP {}", resp.status());
                        return Ok(());
                    }
                    let v: serde_json::Value = resp.json()?;
                    println!("{}", serde_json::to_string_pretty(&v)?);
                }
            }
        }
    }
    Ok(())
}
