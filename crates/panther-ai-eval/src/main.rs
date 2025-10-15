use anyhow::{anyhow, Result};
use clap::Parser;
use panther_validation::{LLMValidator, ProviderFactory, ValidationResult};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(name = "panther-ai-eval", version, author, about = "PantherSDK AI Evaluation CLI")]
struct Cli {
    /// Batch input (JSONL or CSV). If omitted, uses single-run prompt.
    #[arg(short, long)]
    input: Option<PathBuf>,
    /// Providers list JSON (white-label). If omitted, use env-based providers.
    #[arg(short = 'P', long = "providers")]
    providers_path: Option<PathBuf>,
    /// Guidelines JSON path (default to ANVISA bundled)
    #[arg(short, long)]
    guidelines: Option<PathBuf>,
    /// Output directory (default: outputs)
    #[arg(short, long, default_value = "outputs")] 
    out: PathBuf,
    /// Max concurrency for batch (default: 4)
    #[arg(long, default_value = "4")]
    max_concurrency: usize,
    /// Compute and include proof per item
    #[arg(long)]
    with_proof: bool,
    /// Optional positional prompt for single-run mode
    prompt: Vec<String>,
}

#[derive(serde::Deserialize)]
struct ProviderCfg { #[serde(rename = "type")] ty: String, base_url: Option<String>, model: Option<String>, api_key: Option<String> }

#[derive(Debug, serde::Deserialize)]
struct JsonlItem { prompt: String, #[serde(default)] salt: Option<String> }

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    fs::create_dir_all(&cli.out).ok();

    // Providers
    let providers = if let Some(pth) = cli.providers_path.clone() {
        let text = fs::read_to_string(&pth)?;
        let cfgs: Vec<ProviderCfg> = serde_json::from_str(&text)?;
        build_providers_from_cfg(cfgs)?
    } else {
        let mut v = Vec::new();
        if let Ok(p) = ProviderFactory::openai_from_env() { v.push(p); }
        if let Ok(p) = ProviderFactory::ollama_from_env() { v.push(p); }
        if v.is_empty() {
            eprintln!("No providers configured. Use --providers or set env vars for OpenAI/Ollama.");
            std::process::exit(2);
        }
        v
    };

    // Guidelines
    let guides_path = cli.guidelines.clone().unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../panther-validation/guidelines/anvisa.json"));
    let validator = LLMValidator::from_path(&guides_path, providers)?;
    let validator = Arc::new(validator);

    if cli.input.is_some() {
        run_batch(&cli, validator).await
    } else {
        // Single-run
        let prompt = if cli.prompt.is_empty() { "Explain insulin function".to_string() } else { cli.prompt.join(" ") };
        let results = validator.validate(&prompt).await?;
        print_table(&results);
        fs::write(cli.out.join("validation_results.json"), serde_json::to_string_pretty(&results)?)?;
        Ok(())
    }
}

fn build_providers_from_cfg(cfgs: Vec<ProviderCfg>) -> Result<Vec<(String, Arc<dyn panther_domain::ports::LlmProvider>)>> {
    let mut providers: Vec<(String, Arc<dyn panther_domain::ports::LlmProvider>)> = Vec::new();
    for c in cfgs {
        match c.ty.as_str() {
            #[cfg(feature = "openai")]
            "openai" => {
                if let (Some(key), Some(model)) = (c.api_key.clone(), c.model.clone()) {
                    let base = c.base_url.unwrap_or_else(|| "https://api.openai.com".to_string());
                    let p = panther_providers::openai::OpenAiProvider { api_key: key, model: model.clone(), base_url: base };
                    providers.push((format!("openai:{}", model), Arc::new(p)));
                }
            }
            #[cfg(feature = "ollama")]
            "ollama" => {
                if let (Some(base), Some(model)) = (c.base_url.clone(), c.model.clone()) {
                    let p = panther_providers::ollama::OllamaProvider { base_url: base, model: model.clone() };
                    providers.push((format!("ollama:{}", model), Arc::new(p)));
                }
            }
            _ => {}
        }
    }
    if providers.is_empty() { return Err(anyhow!("no providers configured from --providers file")); }
    Ok(providers)
}

async fn run_batch(cli: &Cli, validator: Arc<LLMValidator>) -> Result<()> {
    let input_path = cli.input.as_ref().unwrap();
    let ext = input_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase();
    let mut items: Vec<JsonlItem> = Vec::new();
    if ext == "csv" {
        let mut rdr = csv::Reader::from_path(input_path)?;
        for rec in rdr.deserialize() {
            let row: JsonlItem = rec?;
            items.push(row);
        }
    } else {
        // JSONL default
        let text = fs::read_to_string(input_path)?;
        for (i, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() { continue; }
            let obj: JsonlItem = match serde_json::from_str(line) { Ok(v) => v, Err(e) => { eprintln!("invalid json at line {}: {}", i+1, e); continue; } };
            items.push(obj);
        }
    }
    if items.is_empty() { eprintln!("no items to process"); return Ok(()); }

    let semaphore = Arc::new(tokio::sync::Semaphore::new(cli.max_concurrency.max(1)));
    let mut handles = Vec::new();
    for (idx, it) in items.into_iter().enumerate() {
        let validator = validator.clone();
        let out_dir = cli.out.clone();
        let with_proof = cli.with_proof;
        let providers_json = if let Some(pth) = cli.providers_path.clone() { fs::read_to_string(pth).unwrap_or_else(|_| "[]".to_string()) } else { "[]".to_string() };
        let guidelines_json = fs::read_to_string(cli.guidelines.clone().unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../panther-validation/guidelines/anvisa.json"))).unwrap_or_else(|_| "[]".to_string());
        let permit = semaphore.clone().acquire_owned().await?;
        let handle = tokio::spawn(async move {
            let _permit = permit; // hold until end
            let mut out_obj = serde_json::json!({
                "index": idx,
                "prompt": it.prompt,
            });
            match validator.validate(&out_obj["prompt"].as_str().unwrap()).await {
                Ok(results) => {
                    out_obj["results"] = serde_json::to_value(&results).unwrap_or(serde_json::json!([]));
                    if with_proof {
                        let ctx = panther_validation::proof::ProofContext { sdk_version: env!("CARGO_PKG_VERSION").to_string(), salt: it.salt.clone() };
                        let results_json = serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string());
                        if let Ok(proof) = panther_validation::proof::compute_proof(out_obj["prompt"].as_str().unwrap(), &providers_json, &guidelines_json, &results_json, &ctx) {
                            out_obj["proof"] = serde_json::to_value(proof).unwrap_or(serde_json::json!({}));
                        }
                    }
                }
                Err(e) => {
                    out_obj["error"] = serde_json::json!(e.to_string());
                }
            }
            let line = serde_json::to_string(&out_obj).unwrap_or_else(|_| "{}".to_string());
            let _ = append_line(out_dir.join("results.jsonl"), &line);
        });
        handles.push(handle);
    }
    for h in handles { let _ = h.await; }

    // Summary per provider
    let res_text = fs::read_to_string(cli.out.join("results.jsonl")).unwrap_or_default();
    let mut per: std::collections::HashMap<String, (usize, usize, Vec<i64>)> = std::collections::HashMap::new();
    for line in res_text.lines() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(arr) = v.get("results").and_then(|r| r.as_array()) {
                for r in arr {
                    let prov = r.get("provider_name").and_then(|s| s.as_str()).unwrap_or("").to_string();
                    let lat = r.get("latency_ms").and_then(|n| n.as_i64()).unwrap_or(0);
                    let adher = r.get("adherence_score").and_then(|n| n.as_f64()).unwrap_or(0.0);
                    let e = per.entry(prov).or_insert((0,0,Vec::new()));
                    e.0 += 1;
                    if adher == 0.0 { e.1 += 1; }
                    e.2.push(lat);
                }
            }
        }
    }
    let mut wtr = csv::Writer::from_path(cli.out.join("summary.csv"))?;
    wtr.write_record(["provider","total","errors","p50_ms","p95_ms"]).ok();
    for (prov, (tot, errs, mut lats)) in per {
        lats.sort();
        let idx = |p: f64| -> usize { if lats.is_empty() { 0 } else { ((p * (lats.len() as f64 - 1.0)).round() as isize).clamp(0, lats.len() as isize - 1) as usize } };
        let p50 = lats.get(idx(0.50)).cloned().unwrap_or(0);
        let p95 = lats.get(idx(0.95)).cloned().unwrap_or(0);
        let _ = wtr.write_record(&[prov, tot.to_string(), errs.to_string(), p50.to_string(), p95.to_string()]);
    }
    let _ = wtr.flush();
    println!("\nBatch completed. Artifacts: {}", cli.out.display());
    Ok(())
}

fn append_line(path: PathBuf, line: &str) -> Result<()> {
    use std::io::Write;
    let mut f = if path.exists() { fs::OpenOptions::new().append(true).open(&path)? } else { fs::File::create(&path)? };
    writeln!(f, "{}", line)?;
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
