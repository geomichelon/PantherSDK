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
    /// Comma-separated metrics to compute (e.g., rouge,factcheck)
    #[arg(long)]
    metrics: Option<String>,
    /// USD per 1k tokens (estimate cost for outputs)
    #[arg(long, value_name = "USD_PER_1K")]
    usd_per_1k: Option<f64>,
    /// RAG index (JSONL of {id,text} or directory with .txt files)
    #[arg(long)]
    rag_index: Option<PathBuf>,
    /// RAG k (top-k)
    #[arg(long)]
    rag_k: Option<usize>,
    /// RAG similarity threshold (0..1)
    #[arg(long)]
    rag_threshold: Option<f64>,
    /// RAG chunk sizes (comma-separated, words per chunk; e.g., "50,100"). 0 or empty means no chunking
    #[arg(long)]
    rag_chunk_sizes: Option<String>,
    /// RAG chunk overlap (words)
    #[arg(long)]
    rag_chunk_overlap: Option<usize>,
    /// RAG thresholds (comma-separated) to sweep in experiments (e.g., "0.1,0.3,0.5"). If omitted, uses `--rag-threshold` or 0.0
    #[arg(long)]
    rag_thresholds: Option<String>,
    /// Plagiarism corpus (JSONL with {text} or raw lines; CSV with column `text`; or directory of .txt files)
    #[arg(long)]
    plag_corpus: Option<PathBuf>,
    /// Plagiarism n-gram size (default: 3)
    #[arg(long, default_value = "3")]
    plag_ngram: usize,
    /// Costs file (JSON) with rules per provider/model to estimate costs (USD per 1k tokens in/out)
    #[arg(long)]
    costs: Option<PathBuf>,
    /// Generate advanced HTML report (comparative charts incl. costs)
    #[arg(long)]
    report_advanced_html: bool,
    /// Generate rule-based rewrite for best output per item (rewrites.jsonl)
    #[arg(long)]
    rewrite: bool,
    /// Rewrite style (neutral|formal|simple)
    #[arg(long, default_value = "neutral")]
    rewrite_style: String,
    /// Rewrite locale (en|pt)
    #[arg(long)]
    rewrite_locale: Option<String>,
    /// API base to use API-backed metrics (e.g., http://127.0.0.1:8000)
    #[arg(long)]
    api_base: Option<String>,
    /// API key (X-API-Key) for the backend
    #[arg(long)]
    api_key: Option<String>,
    /// API metric to run on best output per item (factcheck_sources|contextual_relevance|bias_rewrite)
    #[arg(long)]
    api_metric: Option<String>,
    /// API: sources JSON (for factcheck_sources)
    #[arg(long)]
    sources: Option<PathBuf>,
    /// API: per-source thresholds JSON (for factcheck_sources)
    #[arg(long)]
    source_thresholds: Option<PathBuf>,
    /// API: method (jaccard|bow|embed) for factcheck_sources
    #[arg(long)]
    api_method: Option<String>,
    /// API: contradiction method (heuristic|nli) for factcheck_sources
    #[arg(long)]
    contradiction_method: Option<String>,
    /// API: top-k evidences per source
    #[arg(long)]
    evidence_k: Option<usize>,
    /// API: top-k sources
    #[arg(long)]
    top_k: Option<usize>,
    /// API: domain for contextual/rewrite
    #[arg(long)]
    domain: Option<String>,
    /// API: locale for contextual/rewrite
    #[arg(long)]
    locale: Option<String>,
    /// API: target style for rewrite (neutral|formal|simple)
    #[arg(long)]
    target_style: Option<String>,
    /// API: keywords must file (JSON array of strings)
    #[arg(long)]
    keywords_must: Option<PathBuf>,
    /// API: keywords should file (JSON array of strings)
    #[arg(long)]
    keywords_should: Option<PathBuf>,
    /// Variants to apply per prompt (comma-separated: short,detailed,bullets,formal,layman)
    #[arg(long)]
    variants: Option<String>,
    /// Optional positional prompt for single-run mode
    prompt: Vec<String>,
    /// Directory of scenarios (JSONL/CSV). Runs each and aggregates multi-prompt consistency.
    #[arg(long)]
    scenarios: Option<PathBuf>,
    /// Scenarios config (YAML/JSON) with files and variants per scenario
    #[arg(long)]
    scenarios_config: Option<PathBuf>,
    /// Generate HTML report for consistency (`consistency_report.html`)
    #[arg(long)]
    report_html: bool,
}

#[derive(serde::Deserialize)]
struct ProviderCfg { #[serde(rename = "type")] ty: String, base_url: Option<String>, model: Option<String>, api_key: Option<String> }

#[derive(Debug, Clone, serde::Deserialize)]
struct JsonlItem { prompt: String, #[serde(default)] salt: Option<String>, #[serde(default)] labels: Option<Vec<String>> }

#[derive(Debug, serde::Deserialize)]
struct CsvRow { prompt: String, #[serde(default)] salt: Option<String>, #[serde(default)] labels: Option<String> }

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

    if let Some(dir) = cli.scenarios.clone() {
        run_scenarios(&cli, &dir).await
    } else if cli.input.is_some() {
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

async fn run_scenarios(cli: &Cli, dir: &PathBuf) -> Result<()> {
    // Load from config if provided; else list files from dir
    let mut scenario_files: Vec<(PathBuf, Option<Vec<String>>)> = Vec::new();
    if let Some(cfgp) = cli.scenarios_config.clone() {
        let text = fs::read_to_string(&cfgp)?;
        let cfg: serde_json::Value = if cfgp.extension().and_then(|s| s.to_str()) == Some("yaml") || cfgp.extension().and_then(|s| s.to_str())==Some("yml") {
            let v: serde_yaml::Value = serde_yaml::from_str(&text)?; serde_json::to_value(v)?
        } else { serde_json::from_str(&text)? };
        if let Some(arr) = cfg.get("scenarios").and_then(|v| v.as_array()) {
            for it in arr {
                if let Some(f) = it.get("file").and_then(|s| s.as_str()) {
                    let vars = it.get("variants").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect::<Vec<_>>());
                    let p = if PathBuf::from(f).is_absolute() { PathBuf::from(f) } else { dir.join(f) };
                    scenario_files.push((p, vars));
                }
            }
        }
    }
    if scenario_files.is_empty() {
        if dir.is_dir() {
            for e in fs::read_dir(dir)? {
                let p = e?.path();
                if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                    let ext = ext.to_ascii_lowercase();
                    if ext == "jsonl" || ext == "csv" { scenario_files.push((p, None)); }
                }
            }
        } else if dir.is_file() { scenario_files.push((dir.clone(), None)); }
    }
    scenario_files.sort_by(|a,b| a.0.cmp(&b.0));
    if scenario_files.is_empty() { eprintln!("no scenario files"); return Ok(()); }
    // Run each scenario and collect per-provider mean scores from summary_consistency.csv
    let mut per_provider: std::collections::HashMap<String, Vec<(String, f64)>> = std::collections::HashMap::new();
    for (scen, scen_vars) in scenario_files {
        let stem = scen.file_stem().and_then(|s| s.to_str()).unwrap_or("scenario").to_string();
        let out_dir = cli.out.join(&stem);
        fs::create_dir_all(&out_dir).ok();
        // Build a sub-cli
        let sub = Cli {
            input: Some(scen.clone()),
            providers_path: cli.providers_path.clone(),
            guidelines: cli.guidelines.clone(),
            out: out_dir.clone(),
            max_concurrency: cli.max_concurrency,
            with_proof: cli.with_proof,
            metrics: cli.metrics.clone(),
            usd_per_1k: cli.usd_per_1k,
            rag_index: cli.rag_index.clone(),
            rag_k: cli.rag_k,
            rag_threshold: cli.rag_threshold,
            rag_chunk_sizes: cli.rag_chunk_sizes.clone(),
            rag_chunk_overlap: cli.rag_chunk_overlap,
            rag_thresholds: cli.rag_thresholds.clone(),
            prompt: vec![],
            plag_corpus: cli.plag_corpus.clone(),
            plag_ngram: cli.plag_ngram,
            costs: cli.costs.clone(),
            scenarios: None,
            scenarios_config: None,
            report_html: cli.report_html,
            report_advanced_html: cli.report_advanced_html,
            rewrite: cli.rewrite,
            rewrite_style: cli.rewrite_style.clone(),
            rewrite_locale: cli.rewrite_locale.clone(),
            api_base: cli.api_base.clone(),
            api_key: cli.api_key.clone(),
            api_metric: cli.api_metric.clone(),
            sources: cli.sources.clone(),
            source_thresholds: cli.source_thresholds.clone(),
            api_method: cli.api_method.clone(),
            contradiction_method: cli.contradiction_method.clone(),
            evidence_k: cli.evidence_k,
            top_k: cli.top_k,
            domain: cli.domain.clone(),
            locale: cli.locale.clone(),
            target_style: cli.target_style.clone(),
            keywords_must: cli.keywords_must.clone(),
            keywords_should: cli.keywords_should.clone(),
            variants: scen_vars.map(|v| v.join(",")),
        };
        let providers = if let Some(pth) = sub.providers_path.clone() {
            let text = fs::read_to_string(&pth)?;
            let cfgs: Vec<ProviderCfg> = serde_json::from_str(&text)?;
            build_providers_from_cfg(cfgs)?
        } else {
            let mut v = Vec::new();
            if let Ok(p) = ProviderFactory::openai_from_env() { v.push(p); }
            if let Ok(p) = ProviderFactory::ollama_from_env() { v.push(p); }
            if v.is_empty() { eprintln!("No providers configured for scenario {}", stem); continue; }
            v
        };
        let guides_path = sub.guidelines.clone().unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../panther-validation/guidelines/anvisa.json"));
        let validator = LLMValidator::from_path(&guides_path, providers)?;
        let validator = Arc::new(validator);
        run_batch(&sub, validator).await?;
        // read summary_consistency.csv and capture mean_score per provider
        let conc_path = out_dir.join("summary_consistency.csv");
        if conc_path.exists() {
            let mut rdr = csv::Reader::from_path(conc_path)?;
            for rec in rdr.records() {
                let r = rec?;
                let prov = r.get(0).unwrap_or("").to_string();
                let mean_score: f64 = r.get(2).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                per_provider.entry(prov).or_default().push((stem.clone(), mean_score));
            }
        }
    }
    // Aggregate across scenarios: mean/std/cv over scenario mean_score per provider
    let mut w = csv::Writer::from_path(cli.out.join("consistency_multiprompt.csv"))?;
    w.write_record(["provider","n_scenarios","mean_of_means","std_of_means","cv_of_means"]).ok();
    for (prov, arr) in per_provider {
        if arr.is_empty() { continue; }
        let vals = arr.iter().map(|(_,v)| *v).collect::<Vec<_>>();
        let n = vals.len() as f64;
        let mean = vals.iter().sum::<f64>() / n;
        let var = vals.iter().map(|x| (x-mean)*(x-mean)).sum::<f64>() / n.max(1.0);
        let std = var.sqrt();
        let cv = if mean.abs() < 1e-9 { 1.0 } else { (std / mean).abs() };
        w.write_record(&[prov, format!("{}", vals.len()), format!("{:.4}", mean), format!("{:.4}", std), format!("{:.4}", cv)]).ok();
    }
    let _ = w.flush();
    if cli.report_html {
        write_consistency_report_html(&cli.out, &cli.out.join("consistency_multiprompt.csv")).ok();
    }
    Ok(())
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
            let row: CsvRow = rec?;
            let labels = row.labels.as_ref().map(|s| s.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect::<Vec<_>>() ).filter(|v| !v.is_empty());
            items.push(JsonlItem { prompt: row.prompt, salt: row.salt, labels });
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

    let items = expand_variants(items, cli.variants.as_deref());
    let inputs_snapshot = items.clone();
    // Optional: load plagiarism corpus if provided
    let (plag_corpus_json, plag_ngram) = if let Some(p) = cli.plag_corpus.clone() {
        let corpus = load_corpus(&p).unwrap_or_default();
        (Some(serde_json::to_string(&corpus).unwrap_or_else(|_| "[]".to_string())), cli.plag_ngram.max(1))
    } else { (None, cli.plag_ngram.max(1)) };
    let semaphore = Arc::new(tokio::sync::Semaphore::new(cli.max_concurrency.max(1)));
    let mut handles = Vec::new();
    for (idx, it) in items.into_iter().enumerate() {
        let validator = validator.clone();
        let out_dir = cli.out.clone();
        let with_proof = cli.with_proof;
        let metrics = cli.metrics.clone().unwrap_or_default();
        let rewrite_flag = if cli.rewrite { Some("1".to_string()) } else { None };
        if rewrite_flag.is_some() { std::env::set_var("PANTHER_AI_EVAL_REWRITE_FLAG", "1"); }
        if let Some(loc) = cli.rewrite_locale.clone() { std::env::set_var("PANTHER_AI_EVAL_REWRITE_LOCALE", loc); }
        std::env::set_var("PANTHER_AI_EVAL_REWRITE_STYLE", cli.rewrite_style.clone());
        let providers_json = if let Some(pth) = cli.providers_path.clone() { fs::read_to_string(pth).unwrap_or_else(|_| "[]".to_string()) } else { "[]".to_string() };
        let guidelines_json = fs::read_to_string(cli.guidelines.clone().unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../panther-validation/guidelines/anvisa.json"))).unwrap_or_else(|_| "[]".to_string());
        let plag_corpus_json = plag_corpus_json.clone();
        let plag_ngram = plag_ngram;
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
                    // Optional metrics
                    if !metrics.is_empty() {
                        let mut extra = serde_json::Map::new();
                        let text_best = results.iter().max_by(|a,b| a.adherence_score.partial_cmp(&b.adherence_score).unwrap()).map(|r| r.raw_text.clone()).unwrap_or_default();
                        for m in metrics.split(',').map(|s| s.trim().to_lowercase()) {
                            match m.as_str() {
                                "rouge" => {
                                    let r = panthersdk::domain::metrics::evaluate_rouge_l(&out_obj["prompt"].as_str().unwrap(), &text_best);
                                    extra.insert("rouge_l".to_string(), serde_json::json!(r));
                                }
                                "factcheck" => {
                                    // Extract expected terms from guidelines (best-effort)
                                    let facts: Vec<String> = serde_json::from_str::<serde_json::Value>(&guidelines_json)
                                        .ok()
                                        .and_then(|v| v.as_array().cloned())
                                        .map(|arr| arr.into_iter().flat_map(|g| g.get("expected_terms").and_then(|e| e.as_array()).cloned().unwrap_or_default()).filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
                                        .unwrap_or_default();
                                    let score = panthersdk::domain::metrics::evaluate_fact_coverage(&facts, &text_best);
                                    extra.insert("fact_coverage".to_string(), serde_json::json!(score));
                                }
                                "factcheck-adv" | "factcheck_adv" => {
                                    let facts: Vec<String> = serde_json::from_str::<serde_json::Value>(&guidelines_json)
                                        .ok()
                                        .and_then(|v| v.as_array().cloned())
                                        .map(|arr| arr.into_iter().flat_map(|g| g.get("expected_terms").and_then(|e| e.as_array()).cloned().unwrap_or_default()).filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
                                        .unwrap_or_default();
                                    let score = panthersdk::domain::metrics::evaluate_factcheck_adv(&facts, &text_best);
                                    extra.insert("factcheck_adv".to_string(), serde_json::json!(score));
                                }
                                "plagiarism" => {
                                    if let Some(cjson) = plag_corpus_json.as_ref() {
                                        if !text_best.trim().is_empty() {
                                            let corpus: Vec<String> = serde_json::from_str(cjson).unwrap_or_default();
                                            if !corpus.is_empty() {
                                                let score = panthersdk::domain::metrics::evaluate_plagiarism_ngram(&corpus, &text_best, plag_ngram);
                                                extra.insert("plagiarism".to_string(), serde_json::json!(score));
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        if !extra.is_empty() { out_obj["metrics"] = serde_json::Value::Object(extra); }
                    }

                    // Optional rewrite (rule-based) for best output
                    let do_rewrite_env = std::env::var("PANTHER_AI_EVAL_REWRITE").ok().as_deref() == Some("1");
                    let do_rewrite_flag = std::env::var("PANTHER_AI_EVAL_REWRITE_FLAG").ok().as_deref() == Some("1");
                    if do_rewrite_env || do_rewrite_flag {
                        if let Some(best) = results.iter().max_by(|a,b| a.adherence_score.partial_cmp(&b.adherence_score).unwrap()) {
                            let best_text = best.raw_text.clone();
                            let style = std::env::var("PANTHER_AI_EVAL_REWRITE_STYLE").unwrap_or_else(|_| "neutral".to_string());
                            let locale = std::env::var("PANTHER_AI_EVAL_REWRITE_LOCALE").unwrap_or_else(|_| "en".to_string());
                            // Parse must terms from guidelines (best-effort)
                            let must: Vec<String> = serde_json::from_str::<serde_json::Value>(&guidelines_json)
                                .ok()
                                .and_then(|v| v.as_array().cloned())
                                .map(|arr| arr.into_iter().flat_map(|g| g.get("expected_terms").and_then(|e| e.as_array()).cloned().unwrap_or_default()).filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
                                .unwrap_or_default();
                            let rewritten = rewrite_rule(&best_text, &must, &style, &locale);
                            let _ = append_line(out_dir.join("rewrites.jsonl"), &serde_json::json!({
                                "index": idx,
                                "provider": best.provider_name,
                                "style": style,
                                "locale": locale,
                                "original": best_text,
                                "rewritten": rewritten,
                            }).to_string());
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

    // API-backed metrics on best output per item
    if cli.api_base.is_some() && cli.api_metric.is_some() {
        if let Err(e) = run_api_metrics(cli).await { eprintln!("api metrics error: {}", e); }
    }

    // Summary per provider
    let res_text = fs::read_to_string(cli.out.join("results.jsonl")).unwrap_or_default();
    let mut per: std::collections::HashMap<String, (usize, usize, Vec<i64>, usize)> = std::collections::HashMap::new();
    let mut scores_map: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();
    for line in res_text.lines() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(arr) = v.get("results").and_then(|r| r.as_array()) {
                for r in arr {
                    let prov = r.get("provider_name").and_then(|s| s.as_str()).unwrap_or("").to_string();
                    let lat = r.get("latency_ms").and_then(|n| n.as_i64()).unwrap_or(0);
                    let adher = r.get("adherence_score").and_then(|n| n.as_f64()).unwrap_or(0.0);
                    let toks = r.get("raw_text").and_then(|s| s.as_str()).map(|t| t.split_whitespace().count()).unwrap_or(0);
                    let e = per.entry(prov.clone()).or_insert((0,0,Vec::new(),0));
                    e.0 += 1; // total
                    if adher == 0.0 { e.1 += 1; } // errors
                    e.2.push(lat); // latencies
                    e.3 += toks; // tokens
                    scores_map.entry(prov).or_default().push(adher);
                }
            }
        }
    }
    let mut wtr = csv::Writer::from_path(cli.out.join("summary.csv"))?;
    let header = if cli.usd_per_1k.is_some() { vec!["provider","total","errors","p50_ms","p95_ms","avg_tokens","est_cost_usd"] } else { vec!["provider","total","errors","p50_ms","p95_ms","avg_tokens"] };
    wtr.write_record(header).ok();
    for (prov, (tot, errs, lats, toks)) in &per {
        let mut lats = lats.clone();
        lats.sort();
        let idx = |p: f64| -> usize { if lats.is_empty() { 0 } else { ((p * (lats.len() as f64 - 1.0)).round() as isize).clamp(0, lats.len() as isize - 1) as usize } };
        let p50 = lats.get(idx(0.50)).cloned().unwrap_or(0);
        let p95 = lats.get(idx(0.95)).cloned().unwrap_or(0);
        let avg_tokens = if *tot > 0 { (*toks as f64 / *tot as f64).round() as i64 } else { 0 };
        if let Some(usd_per_1k) = cli.usd_per_1k {
            let est_cost = (*toks as f64 / 1000.0) * usd_per_1k;
            let _ = wtr.write_record(&[prov.clone(), tot.to_string(), errs.to_string(), p50.to_string(), p95.to_string(), avg_tokens.to_string(), format!("{:.4}", est_cost)]);
        } else {
            let _ = wtr.write_record(&[prov.clone(), tot.to_string(), errs.to_string(), p50.to_string(), p95.to_string(), avg_tokens.to_string()]);
        }
    }
    let _ = wtr.flush();
    // Advanced analysis and comparative report (incl. costs)
    if let Err(e) = generate_advanced_reports(cli) { eprintln!("advanced report error: {}", e); }

    // Optional: RAG evaluation if index provided
    if let Some(index_path) = cli.rag_index.clone() {
        run_rag_eval(&cli, &inputs_snapshot, &index_path).await.ok();
        run_rag_experiments(&cli, &inputs_snapshot, &index_path).await.ok();
    }

    // Consistency summary per provider (scores across prompts)
    let mut wtr2 = csv::Writer::from_path(cli.out.join("summary_consistency.csv"))?;
    wtr2.write_record(["provider","n","mean_score","std_score","cv_score","consistency_index","mean_latency_ms","std_latency_ms"]).ok();
    for (prov, scores) in &scores_map {
        let n = scores.len() as f64;
        if n == 0.0 { continue; }
        let mean = scores.iter().copied().sum::<f64>() / n;
        let var = scores.iter().map(|s| (s - mean)*(s - mean)).sum::<f64>() / n.max(1.0);
        let std = var.sqrt();
        let cv = if mean.abs() < 1e-9 { 1.0 } else { (std / mean).abs() };
        let consistency_index = (1.0 - cv).clamp(0.0, 1.0);
        // latency stats reuse from per map
        let (mean_lat, std_lat) = if let Some((_tot, _errs, lvec, _tok)) = per.get(prov) {
            let lvec = lvec.clone();
            if lvec.is_empty() { (0.0, 0.0) } else {
                let n2 = lvec.len() as f64;
                let m2 = lvec.iter().map(|v| *v as f64).sum::<f64>() / n2;
                let var2 = lvec.iter().map(|v| { let x = *v as f64 - m2; x*x }).sum::<f64>() / n2;
                (m2, var2.sqrt())
            }
        } else { (0.0, 0.0) };
        let _ = wtr2.write_record(&[
            prov.to_string(),
            (scores.len()).to_string(),
            format!("{:.4}", mean),
            format!("{:.4}", std),
            format!("{:.4}", cv),
            format!("{:.4}", consistency_index),
            format!("{:.0}", mean_lat),
            format!("{:.0}", std_lat),
        ]);
    }
    let _ = wtr2.flush();
    println!("\nBatch completed. Artifacts: {}", cli.out.display());
    Ok(())
}

async fn run_api_metrics(cli: &Cli) -> Result<()> {
    use reqwest::Client;
    let base = cli.api_base.clone().unwrap();
    let key = cli.api_key.clone().unwrap_or_default();
    let contents = fs::read_to_string(cli.out.join("results.jsonl")).unwrap_or_default();
    let client = Client::builder().build()?;
    let metric = cli.api_metric.clone().unwrap().to_lowercase();
    for line in contents.lines() {
        let v: serde_json::Value = match serde_json::from_str(line) { Ok(x) => x, Err(_) => continue };
        let best_text = v.get("results").and_then(|r| r.as_array()).and_then(|arr| {
            arr.iter().max_by(|a,b| a.get("adherence_score").and_then(|n| n.as_f64()).unwrap_or(0.0).partial_cmp(&b.get("adherence_score").and_then(|n| n.as_f64()).unwrap_or(0.0)).unwrap()).cloned()
        }).and_then(|r| r.get("raw_text").and_then(|s| s.as_str()).map(|s| s.to_string() ));
        let Some(text_best) = best_text else { continue };
        let url = format!("{}/metrics/evaluate", base.trim_end_matches('/'));
        // Build body per metric
        let mut body = serde_json::json!({"metric": metric, "text": text_best});
        match metric.as_str() {
            "factcheck_sources" => {
                // sources
                if let Some(p) = cli.sources.clone() {
                    if let Ok(s) = fs::read_to_string(p) { if let Ok(j) = serde_json::from_str::<serde_json::Value>(&s) { body["sources"] = j; } }
                }
                if let Some(p) = cli.source_thresholds.clone() {
                    if let Ok(s) = fs::read_to_string(p) { if let Ok(j) = serde_json::from_str::<serde_json::Value>(&s) { body["source_thresholds"] = j; } }
                }
                if let Some(m) = cli.api_method.clone() { body["method"] = serde_json::json!(m); }
                if let Some(cm) = cli.contradiction_method.clone() { body["contradiction_method"] = serde_json::json!(cm); }
                if let Some(k) = cli.top_k { body["top_k"] = serde_json::json!(k); }
                if let Some(ek) = cli.evidence_k { body["evidence_k"] = serde_json::json!(ek); }
            }
            "contextual_relevance" => {
                if let Some(d) = cli.domain.clone() { body["domain"] = serde_json::json!(d); }
                if let Some(l) = cli.locale.clone() { body["locale"] = serde_json::json!(l); }
                if let Some(p) = cli.keywords_must.clone() {
                    if let Ok(s) = fs::read_to_string(p) { if let Ok(j) = serde_json::from_str::<serde_json::Value>(&s) { body["keywords_must"] = j; } }
                }
                if let Some(p) = cli.keywords_should.clone() {
                    if let Ok(s) = fs::read_to_string(p) { if let Ok(j) = serde_json::from_str::<serde_json::Value>(&s) { body["keywords_should"] = j; } }
                }
            }
            "bias_rewrite" => {
                if let Some(d) = cli.domain.clone() { body["domain"] = serde_json::json!(d); }
                if let Some(l) = cli.locale.clone() { body["locale"] = serde_json::json!(l); }
                if let Some(s) = cli.target_style.clone() { body["target_style"] = serde_json::json!(s); }
                body["rewrite_method"] = serde_json::json!("llm");
            }
            _ => {}
        }
        let mut req = client.post(&url).json(&body);
        if !key.is_empty() { req = req.header("X-API-Key", key.clone()); }
        if let Ok(resp) = req.send().await {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                let fname = match metric.as_str() {
                    "factcheck_sources" => "factcheck_results.jsonl",
                    "contextual_relevance" => "contextual_results.jsonl",
                    "bias_rewrite" => "rewrites_api.jsonl",
                    _ => "api_results.jsonl",
                };
                let _ = append_line(cli.out.join(fname), &json.to_string());
            }
        }
    }
    Ok(())
}

#[derive(serde::Deserialize)]
struct RagDoc { id: String, text: String }

fn tokenize_lower(s: &str) -> Vec<String> { s.split_whitespace().map(|t| t.to_ascii_lowercase()).collect() }

fn tf_map(tokens: &[String]) -> std::collections::HashMap<String, f64> {
    let mut m = std::collections::HashMap::new();
    for t in tokens { *m.entry(t.clone()).or_insert(0.0) += 1.0; }
    let norm: f64 = m.values().sum::<f64>().max(1.0);
    for v in m.values_mut() { *v /= norm; }
    m
}

fn cosine(a: &std::collections::HashMap<String,f64>, b: &std::collections::HashMap<String,f64>) -> f64 {
    let mut dot = 0.0; for (k,va) in a { if let Some(vb)= b.get(k) { dot += va*vb; } }
    let na = a.values().map(|v| v*v).sum::<f64>().sqrt();
    let nb = b.values().map(|v| v*v).sum::<f64>().sqrt();
    if na == 0.0 || nb == 0.0 { 0.0 } else { dot / (na*nb) }
}

async fn run_rag_eval(cli: &Cli, items: &Vec<JsonlItem>, index_path: &PathBuf) -> Result<()> {
    // Load index (JSONL with {id,text}) or directory of .txt files
    let mut docs: Vec<RagDoc> = Vec::new();
    if index_path.is_file() {
        let text = fs::read_to_string(index_path)?;
        for line in text.lines() { if line.trim().is_empty() { continue; } if let Ok(d) = serde_json::from_str::<RagDoc>(line) { docs.push(d); } }
    } else if index_path.is_dir() {
        for entry in fs::read_dir(index_path)? { let e = entry?; if e.path().extension().and_then(|s| s.to_str()) == Some("txt") { let id = e.file_name().to_string_lossy().to_string(); let text = fs::read_to_string(e.path())?; docs.push(RagDoc{ id, text }); } }
    }
    if docs.is_empty() { return Ok(()); }
    // Build vectors
    let mut doc_vecs: Vec<(String, std::collections::HashMap<String,f64>)> = Vec::new();
    for d in &docs { let tf = tf_map(&tokenize_lower(&d.text)); doc_vecs.push((d.id.clone(), tf)); }

    let k = cli.rag_k.unwrap_or(5).max(1);
    let threshold = cli.rag_threshold.unwrap_or(0.0);
    let mut out_lines = Vec::new();
    let mut sum_prec = 0.0; let mut sum_rec = 0.0; let mut cnt = 0.0;
    for it in items {
        if it.labels.as_ref().map(|v| v.is_empty()).unwrap_or(true) { continue; }
        let labels = it.labels.as_ref().unwrap();
        let qvec = tf_map(&tokenize_lower(&it.prompt));
        let mut scored: Vec<(String, f64)> = doc_vecs.iter().map(|(id,vec)| (id.clone(), cosine(&qvec, vec))).collect();
        scored.sort_by(|a,b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let top: Vec<(String,f64)> = scored.into_iter().filter(|(_,s)| *s >= threshold).take(k).collect();
        let top_ids: std::collections::HashSet<_> = top.iter().map(|(id,_)| id.clone()).collect();
        let label_set: std::collections::HashSet<_> = labels.iter().cloned().collect();
        let relevant = top_ids.intersection(&label_set).count() as f64;
        let prec = if k > 0 { relevant / (top.len().max(1) as f64) } else { 0.0 };
        let rec = relevant / (label_set.len().max(1) as f64);
        sum_prec += prec; sum_rec += rec; cnt += 1.0;
        out_lines.push(serde_json::json!({
            "prompt": it.prompt,
            "labels": labels,
            "retrieved": top.iter().map(|(id,score)| serde_json::json!({"id": id, "score": score})).collect::<Vec<_>>()
        }));
    }
    // Write per-item results and summary
    let _ = append_line(cli.out.join("rag_results.jsonl"), &out_lines.into_iter().map(|v| v.to_string()).collect::<Vec<_>>().join("\n"));
    let mut w = csv::Writer::from_path(cli.out.join("rag_summary.csv"))?;
    w.write_record(["k","threshold","avg_precision","avg_recall","n"]).ok();
    w.write_record(&[k.to_string(), format!("{:.3}", threshold), format!("{:.4}", if cnt>0.0 { sum_prec/cnt } else {0.0}), format!("{:.4}", if cnt>0.0 { sum_rec/cnt } else {0.0}), format!("{:.0}", cnt)]).ok();
    let _ = w.flush();
    Ok(())
}

fn chunk_words(text: &str, size: usize, overlap: usize) -> Vec<String> {
    if size == 0 { return vec![text.to_string()]; }
    let toks: Vec<&str> = text.split_whitespace().collect();
    if toks.is_empty() { return vec![]; }
    let mut chunks = Vec::new();
    let mut i = 0usize;
    while i < toks.len() {
        let end = (i+size).min(toks.len());
        let ch = toks[i..end].join(" ");
        chunks.push(ch);
        if end == toks.len() { break; }
        i = i + size.saturating_sub(overlap).max(1);
    }
    chunks
}

async fn run_rag_experiments(cli: &Cli, items: &Vec<JsonlItem>, index_path: &PathBuf) -> Result<()> {
    // Load base docs
    let mut docs: Vec<RagDoc> = Vec::new();
    if index_path.is_file() {
        let text = fs::read_to_string(index_path)?;
        for line in text.lines() { if line.trim().is_empty() { continue; } if let Ok(d) = serde_json::from_str::<RagDoc>(line) { docs.push(d); } }
    } else if index_path.is_dir() {
        for entry in fs::read_dir(index_path)? { let e = entry?; if e.path().extension().and_then(|s| s.to_str()) == Some("txt") { let id = e.file_name().to_string_lossy().to_string(); let text = fs::read_to_string(e.path())?; docs.push(RagDoc{ id, text }); } }
    }
    if docs.is_empty() { return Ok(()); }

    // Parse chunk sizes and thresholds
    let chunk_sizes: Vec<usize> = cli.rag_chunk_sizes.as_ref()
        .map(|s| s.split(',').filter_map(|t| t.trim().parse::<usize>().ok()).collect())
        .unwrap_or_else(|| vec![0]);
    let thresholds: Vec<f64> = if let Some(s) = &cli.rag_thresholds {
        s.split(',').filter_map(|t| t.trim().parse::<f64>().ok()).collect()
    } else if let Some(v) = cli.rag_threshold {
        vec![v]
    } else { vec![0.0] };
    let overlap = cli.rag_chunk_overlap.unwrap_or(0);
    let k = cli.rag_k.unwrap_or(5).max(1);

    let mut w = csv::Writer::from_path(cli.out.join("rag_experiments.csv"))?;
    w.write_record(["chunk_size","overlap","k","threshold","avg_precision","avg_recall","avg_f1","mrr","n"]).ok();

    for &sz in &chunk_sizes {
        // Build chunked vectors
        let mut chunk_vecs: Vec<(String, String, std::collections::HashMap<String,f64>)> = Vec::new();
        for d in &docs {
            let parts = chunk_words(&d.text, sz, overlap);
            if parts.is_empty() { continue; }
            for (i,ch) in parts.iter().enumerate() {
                let id = format!("{}#{}", d.id, i);
                chunk_vecs.push((d.id.clone(), id, tf_map(&tokenize_lower(ch))));
            }
        }
        if chunk_vecs.is_empty() { continue; }

        for &th in &thresholds {
            let mut sum_prec = 0.0; let mut sum_rec = 0.0; let mut sum_f1 = 0.0; let mut sum_mrr = 0.0; let mut cnt = 0.0;
            for it in items {
                if it.labels.as_ref().map(|v| v.is_empty()).unwrap_or(true) { continue; }
                let labels = it.labels.as_ref().unwrap();
                let label_set: std::collections::HashSet<_> = labels.iter().cloned().collect();
                let qvec = tf_map(&tokenize_lower(&it.prompt));
                // rank chunks
                let mut scored: Vec<(&String, &String, f64)> = chunk_vecs.iter().map(|(doc,chunk,vec)| (doc, chunk, cosine(&qvec, vec))).collect();
                scored.sort_by(|a,b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
                let mut top_docs = Vec::new();
                let mut seen_docs = std::collections::HashSet::new();
                for (doc, _chunk, s) in scored.into_iter() {
                    if s < th { break; }
                    if seen_docs.insert(doc.clone()) { top_docs.push(doc.clone()); }
                    if top_docs.len() >= k { break; }
                }
                if top_docs.is_empty() { continue; }
                let relevant = top_docs.iter().filter(|d| label_set.contains(&***d)).count() as f64;
                let prec = relevant / (top_docs.len() as f64);
                let rec = relevant / (label_set.len().max(1) as f64);
                let f1 = if prec+rec > 0.0 { 2.0*prec*rec/(prec+rec) } else { 0.0 };
                let mut rr = 0.0;
                for (rank, d) in top_docs.iter().enumerate() {
                    if label_set.contains(&**d) { rr = 1.0 / ((rank+1) as f64); break; }
                }
                sum_prec += prec; sum_rec += rec; sum_f1 += f1; sum_mrr += rr; cnt += 1.0;
            }
            if cnt > 0.0 {
                w.write_record(&[
                    sz.to_string(), overlap.to_string(), k.to_string(), format!("{:.3}", th),
                    format!("{:.4}", sum_prec/cnt), format!("{:.4}", sum_rec/cnt), format!("{:.4}", sum_f1/cnt), format!("{:.4}", sum_mrr/cnt), format!("{:.0}", cnt),
                ]).ok();
            }
        }
    }
    let _ = w.flush();
    Ok(())
}

fn append_line(path: PathBuf, line: &str) -> Result<()> {
    use std::io::Write;
    let mut f = if path.exists() { fs::OpenOptions::new().append(true).open(&path)? } else { fs::File::create(&path)? };
    writeln!(f, "{}", line)?;
    Ok(())
}

fn apply_variant(prompt: &str, var: &str) -> String {
    match var.to_ascii_lowercase().as_str() {
        "short" => format!("In one concise sentence: {}", prompt),
        "detailed" => format!("Explain thoroughly and step-by-step: {}", prompt),
        "bullets" => format!("Answer with short bullet points only: {}", prompt),
        "formal" => format!("Explain formally and academically: {}", prompt),
        "layman" => format!("Explain in simple layman terms: {}", prompt),
        _ => prompt.to_string(),
    }
}

fn expand_variants(items: Vec<JsonlItem>, variants: Option<&str>) -> Vec<JsonlItem> {
    let mut out = Vec::new();
    let vars: Vec<String> = variants
        .map(|s| s.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect())
        .unwrap_or_default();
    if vars.is_empty() { return items; }
    for it in items {
        // include original
        out.push(it.clone());
        for v in &vars {
            let p2 = apply_variant(&it.prompt, v);
            out.push(JsonlItem { prompt: p2, salt: it.salt.clone(), labels: it.labels.clone() });
        }
    }
    out
}

fn write_consistency_report_html(out_dir: &PathBuf, csv_path: &PathBuf) -> Result<()> {
    let html_path = out_dir.join("consistency_report.html");
    let csv_raw = fs::read_to_string(csv_path).unwrap_or_default();
    // Escape backticks and closing tags for safe embedding inside a JS template string
    let csv_escaped = csv_raw.replace('`', "\\`").replace("</", "<\\/");
    let mut s = String::new();
    s.push_str("<!doctype html>\n<html><head><meta charset='utf-8'><title>Consistency Report</title>\n");
    s.push_str("<style>body{font-family:Arial,sans-serif;margin:20px} .bar{background:#4e79a7;height:24px;margin:4px 0;color:#fff;padding-left:6px;line-height:24px}</style>\n");
    s.push_str("</head><body>\n<h2>Consistency Report</h2>\n<p>Data: ");
    s.push_str(&format!("{}", csv_path.display()));
    s.push_str("</p>\n<div id='chart'></div>\n<script>\n");
    s.push_str("const csv = `");
    s.push_str(&csv_escaped);
    s.push_str("`.trim().split(\n).map(l=>l.split(\",\"));\n");
    s.push_str("// header: provider,n,mean,std,cv\n");
    s.push_str("const rows = csv.slice(1).map(r=>({prov:r[0], n:parseInt(r[1]||'0'), mean:parseFloat(r[2]||'0'), std:parseFloat(r[3]||'0'), cv:parseFloat(r[4]||'0')})).sort((a,b)=>b.mean-a.mean);\n");
    s.push_str("const maxMean = Math.max(1, ...rows.map(r=>r.mean));\n");
    s.push_str("const root = document.getElementById('chart');\n");
    s.push_str("rows.forEach(r=>{\n  const div=document.createElement('div'); div.className='bar';\n  div.style.width=(r.mean/maxMean*90+10)+'%';\n  div.textContent=`${r.prov} — mean ${r.mean.toFixed(2)} (n=${r.n})`;\n  root.appendChild(div);\n});\n");
    s.push_str("</script>\n</body></html>");
    fs::write(html_path, s)?;
    Ok(())
}

fn load_corpus(path: &PathBuf) -> Result<Vec<String>> {
    let mut corpus: Vec<String> = Vec::new();
    if path.is_file() {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase();
        if ext == "csv" {
            let mut rdr = csv::Reader::from_path(path)?;
            let headers = rdr.headers()?.clone();
            // choose column index: prefer "text", then "content", else first
            let mut col_idx = 0usize;
            for (i, h) in headers.iter().enumerate() {
                if h.eq_ignore_ascii_case("text") { col_idx = i; break; }
                if h.eq_ignore_ascii_case("content") { col_idx = i; }
            }
            for rec in rdr.records() {
                let r = rec?;
                if let Some(v) = r.get(col_idx) { corpus.push(v.to_string()); }
            }
        } else {
            let text = std::fs::read_to_string(path)?;
            for line in text.lines() {
                let l = line.trim(); if l.is_empty() { continue; }
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(l) {
                    if let Some(s) = v.get("text").and_then(|x| x.as_str()) { corpus.push(s.to_string()); }
                    else if let Some(s) = v.as_str() { corpus.push(s.to_string()); }
                } else {
                    corpus.push(l.to_string());
                }
            }
        }
    } else if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let e = entry?; let p = e.path();
            if p.extension().and_then(|s| s.to_str()) == Some("txt") {
                corpus.push(std::fs::read_to_string(p)?);
            }
        }
    }
    Ok(corpus)
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

fn rewrite_rule(text: &str, must: &[String], style: &str, locale: &str) -> String {
    // Neutralize pronouns (simple)
    let mut t = format!(" {} ", text);
    if locale.to_ascii_lowercase().starts_with("pt") {
        t = t.replace(" ele ", " elu ").replace(" ela ", " elu ").replace(" dele ", " delu ").replace(" dela ", " delu ");
    } else {
        t = t.replace(" he ", " they ").replace(" she ", " they ").replace(" his ", " their ").replace(" her ", " their ");
    }
    t = t.trim().to_string();
    // Enrich with missing must (as gentle suggestion)
    let mut missing: Vec<String> = Vec::new();
    for k in must { if !k.is_empty() && !t.to_ascii_lowercase().contains(&k.to_ascii_lowercase()) { missing.push(k.clone()); } }
    if !missing.is_empty() {
        if locale.to_ascii_lowercase().starts_with("pt") {
            t.push_str(&format!("\n\n(Sugerido) Incluir termos: {}", missing.join(", ")));
        } else {
            t.push_str(&format!("\n\n(Suggested) Include terms: {}", missing.join(", ")));
        }
    }
    if style.eq_ignore_ascii_case("formal") {
        if !t.is_empty() {
            let mut chars = t.chars();
            if let Some(c0) = chars.next() {
                let rest: String = chars.collect();
                let cap = c0.to_uppercase().collect::<String>();
                if locale.to_ascii_lowercase().starts_with("pt") {
                    t = format!("Observação: {}{}", cap, rest);
                } else {
                    t = format!("Please note: {}{}", cap, rest);
                }
            }
        }
    }
    t
}
#[derive(serde::Deserialize)]
struct CostRule { provider: String, usd_per_1k_in: Option<f64>, usd_per_1k_out: Option<f64> }

fn match_rule<'a>(prov: &str, rules: &'a [CostRule]) -> Option<&'a CostRule> {
    let mut best: Option<&CostRule> = None;
    for r in rules {
        let p = r.provider.as_str();
        if p == prov { return Some(r); }
        if p.ends_with(':') && prov.starts_with(p) { best = Some(r); }
        if p == "*" { best = best.or(Some(r)); }
    }
    best
}

fn generate_advanced_reports(cli: &Cli) -> Result<()> {
    use std::collections::HashMap;
    let res_path = cli.out.join("results.jsonl");
    if !res_path.exists() { return Ok(()); }
    let text = fs::read_to_string(&res_path).unwrap_or_default();
    let mut scores: HashMap<String, Vec<f64>> = HashMap::new();
    let mut lats: HashMap<String, Vec<i64>> = HashMap::new();
    let mut errc: HashMap<String, usize> = HashMap::new();
    let mut tok_in: HashMap<String, usize> = HashMap::new();
    let mut tok_out: HashMap<String, usize> = HashMap::new();
    for line in text.lines() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            let prompt_wc = v.get("prompt").and_then(|s| s.as_str()).map(|p| p.split_whitespace().count()).unwrap_or(0);
            if let Some(arr) = v.get("results").and_then(|r| r.as_array()) {
                for r in arr {
                    let prov = r.get("provider_name").and_then(|s| s.as_str()).unwrap_or("").to_string();
                    let sc = r.get("adherence_score").and_then(|n| n.as_f64()).unwrap_or(0.0);
                    let lat = r.get("latency_ms").and_then(|n| n.as_i64()).unwrap_or(0);
                    let out_wc = r.get("raw_text").and_then(|s| s.as_str()).map(|t| t.split_whitespace().count()).unwrap_or(0);
                    scores.entry(prov.clone()).or_default().push(sc);
                    lats.entry(prov.clone()).or_default().push(lat);
                    if sc == 0.0 { *errc.entry(prov.clone()).or_insert(0) += 1; }
                    *tok_in.entry(prov.clone()).or_insert(0) += prompt_wc;
                    *tok_out.entry(prov.clone()).or_insert(0) += out_wc;
                }
            }
        }
    }
    // Load cost rules
    let mut rules: Vec<CostRule> = Vec::new();
    if let Some(p) = cli.costs.clone() {
        if let Ok(s) = fs::read_to_string(p) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                if let Some(arr) = v.as_array() { rules = arr.iter().filter_map(|x| serde_json::from_value::<CostRule>(x.clone()).ok()).collect(); }
            }
        }
    }
    let mut w = csv::Writer::from_path(cli.out.join("advanced_summary.csv"))?;
    w.write_record(["provider","total","errors","mean_score","std_score","cv_score","p50_ms","p95_ms","tokens_in","tokens_out","est_cost_usd"]).ok();
    // Prepare HTML rows
    let mut html_rows: Vec<(String, f64, f64)> = Vec::new();
    for (prov, v) in &scores {
        let vals = v.clone();
        let n = vals.len() as f64;
        if n == 0.0 { continue; }
        let mean = vals.iter().sum::<f64>() / n;
        let var = vals.iter().map(|x| (x-mean)*(x-mean)).sum::<f64>() / n.max(1.0);
        let std = var.sqrt();
        let cv = if mean.abs() < 1e-9 { 1.0 } else { (std/mean).abs() };
        let mut latv = lats.get(prov).cloned().unwrap_or_default();
        latv.sort();
        let idx = |p: f64| -> usize { if latv.is_empty() {0} else {((p * (latv.len() as f64 - 1.0)).round() as isize).clamp(0, latv.len() as isize - 1) as usize}};
        let p50 = *latv.get(idx(0.50)).unwrap_or(&0);
        let p95 = *latv.get(idx(0.95)).unwrap_or(&0);
        let t_in = *tok_in.get(prov).unwrap_or(&0) as f64;
        let t_out = *tok_out.get(prov).unwrap_or(&0) as f64;
        let mut cost = 0.0;
        if !rules.is_empty() {
            if let Some(rule) = match_rule(prov, &rules) {
                let cin = rule.usd_per_1k_in.unwrap_or(0.0);
                let cout = rule.usd_per_1k_out.unwrap_or(0.0);
                cost = (t_in/1000.0)*cin + (t_out/1000.0)*cout;
            }
        } else if let Some(usd) = cli.usd_per_1k { cost = (t_out/1000.0) * usd; }
        w.write_record(&[prov.clone(), format!("{}", v.len()), format!("{}", errc.get(prov).copied().unwrap_or(0)), format!("{:.4}", mean), format!("{:.4}", std), format!("{:.4}", cv), format!("{}", p50), format!("{}", p95), format!("{:.0}", t_in), format!("{:.0}", t_out), format!("{:.4}", cost)]).ok();
        html_rows.push((prov.clone(), mean, cost));
    }
    let _ = w.flush();
    if cli.report_advanced_html {
        let html_path = cli.out.join("advanced_report.html");
        let mut s = String::new();
        s.push_str("<!doctype html><html><head><meta charset='utf-8'><title>Advanced Report</title><style>body{font-family:Arial;margin:20px} .row{margin:6px 0} .bar{display:inline-block;height:20px;background:#4e79a7;margin-right:8px} .bar2{display:inline-block;height:20px;background:#f28e2b;margin-left:12px}</style></head><body>");
        s.push_str("<h2>Advanced Provider Report</h2><p>Mean score (blue) and estimated cost (orange). Cost scale is relative to max.</p>");
        let max_cost = html_rows.iter().map(|(_,_,c)| *c).fold(0.0, f64::max).max(1e-9);
        for (prov, mean, cost) in html_rows {
            let w1 = (mean.max(0.0).min(100.0))/100.0*300.0;
            let w2 = (cost/max_cost*300.0).min(300.0);
            s.push_str(&format!("<div class='row'><span style='display:inline-block;width:140px'>{}</span><span class='bar' style='width:{:.0}px'></span>{:.1}% <span class='bar2' style='width:{:.0}px'></span>${:.2}</div>", prov, w1, mean, w2, cost));
        }
        s.push_str("</body></html>");
        fs::write(html_path, s).ok();
    }
    Ok(())
}
