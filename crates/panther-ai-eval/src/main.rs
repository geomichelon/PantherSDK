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
    /// Optional positional prompt for single-run mode
    prompt: Vec<String>,
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
