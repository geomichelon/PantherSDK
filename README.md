PantherSDK (Rust core + Python API)

PantherSDK is a cross-platform SDK for evaluation, analysis, and integration of generative AI (LLMs). It uses a Rust core and a Python API layer, following Hexagonal (Clean) Architecture.

Goals
- Unified base for LLM evaluation, analysis, and integration
- Usable from iOS, Android, Flutter, and React Native
- Automatic bindings across platforms via stable FFI or generators (PyO3 for Python, wasm-bindgen for JS, UniFFI optional for Swift/Kotlin)
- Auditing, metrics, and enterprise integrations

FAQ
- Consulte docs/FAQ.md para respostas rápidas sobre cada feature (validação, proof, guidelines similarity, embeddings, storage/logs, custos, métricas, iOS build, etc.).

What’s Included (PoC summary)
- Multi‑provider validation (OpenAI/Ollama/Anthropic via features), ranking by adherence, latency tracking, token/cost estimation
- Proof generation/verification and optional anchoring on Ethereum (API helper)
- Guidelines Similarity (FFI): ingest JSON (http/s3/gs) and compute similarity by BOW/Jaccard/Hybrid; optional embeddings (OpenAI/Ollama) with cosine
- Storage helpers (in‑memory/sled) and log capture for audit trails
- Bias and content metrics (BLEU, ROUGE‑L, accuracy, fluency, diversity, plagiarism)
- Samples for iOS (Swift), Android (Kotlin), Flutter (Dart FFI) e React Native (Android/iOS)

iOS Sample — Tabs Overview
- LLMs
  - Selecione e configure os LLMs (OpenAI/Ollama/Anthropic) usados na validação (Single).
  - “Salvar sessão” persiste as credenciais/modelos no `UserDefaults`.
  - “LLMs (JSON avançado)”: edite manualmente a lista usada em Multi/With Proof. Exemplo (OpenAI chatgpt‑5 + gpt‑4o):
    ```json
    [
      { "type": "openai", "api_key": "sk-…", "base_url": "https://api.openai.com", "model": "chatgpt-5" },
      { "type": "openai", "api_key": "sk-…", "base_url": "https://api.openai.com", "model": "gpt-4o" }
    ]
    ```
  - Estimativa de custos: “Editar Tabela” abre um editor da tabela de preços por modelo (utilizada para estimar custo por resultado).
  - ![Providers](docs/images/providers.png)

- Validate
  - Prompt + modo de execução:
    - Single: executa somente o provider selecionado na aba Providers.
    - Multi: executa em paralelo todos os LLMs do JSON avançado (se fornecido). Caso contrário, usa apenas o LLM selecionado.
    - With Proof: igual ao Multi, mas retorna também um objeto `proof` (hash combinado) para auditoria/âncora.
  - Diretrizes: opcionalmente use um JSON customizado (senão usa ANVISA embutido). Os modos Multi/Proof também possuem variantes `validateCustom*` quando diretrizes customizadas estão ativas.
  - Similarity (na mesma tela): carregar diretrizes por URL, salvar/carregar um índice nomeado e calcular “similarity” (BOW/Jaccard/Hybrid/Embeddings) com `topK`.
  - Tokens e custo: a tela estima custo por provider com base na contagem de tokens e na tabela de preços.
  - ![Validate](docs/images/validate.png)

Observação sobre o sample
- Para Multi-provider, utilize “LLMs (JSON avançado)” na aba LLMs para compor a lista.

Repository Structure
- `Cargo.toml` (Rust workspace)
- `crates/`
  - `panther-domain` — entities, ports (contracts), and errors
  - `panther-core` — use cases and orchestration
  - `panther-providers` — adapters for providers (e.g., OpenAI, Local)
  - `panther-observability` — tracing/logging/telemetry
  - `panther-metrics` — metrics adapters (e.g., in-memory; Prometheus later)
  - `panther-storage` — storage adapters (e.g., in-memory; Redis/Postgres later)
  - `panther-ffi` — stable C ABI for bindings (Swift/Kotlin/Dart/RN)
- `python/panthersdk/` — API layer (FastAPI) and enterprise integrations
- `crates/panther-py` — PyO3 extension module for Python (built via maturin)
- `bindings/` — instructions and header/wrapper generation
- `docs/` — architecture, decisions, and guides

Key FFI Exports (C ABI)
- Core: `panther_init`, `panther_generate`, `panther_version_string`, `panther_free_string`
- Validation: `panther_validation_run_default|openai|ollama|multi|custom`, plus `*_with_proof`
- Proof: `panther_proof_compute`, `panther_proof_verify_local`, `panther_proof_anchor_eth`, `panther_proof_check_eth`
- Metrics/Storage/Logs: `panther_metrics_*`, `panther_storage_*`, `panther_logs_*`, `panther_token_count`, `panther_calculate_cost`
- Guidelines Similarity:
  - `panther_guidelines_ingest_json(guidelines_json)` — indexa diretrizes (array de objetos `{topic, expected_terms[]}` ou `string[]`)
  - `panther_guidelines_similarity(query, top_k, method)` — `method`: `bow|jaccard|hybrid|embed-openai|embed-ollama`
  - `panther_guidelines_save_json(name, json)` / `panther_guidelines_load(name)` — persistência simples (via storage*)
  - `panther_guidelines_embeddings_build(method)` — pré‑cálculo vetorial (OpenAI/Ollama)

Bindings Path
- Stable C ABI (via `panther-ffi`) + `cbindgen` to generate `panther.h`
- iOS (Swift): wrapper calling the C ABI
- Android (Kotlin/JNI): wrapper calling the C ABI
- Flutter (Dart FFI): direct C ABI calls
- React Native (TurboModule/JSI): C++/JNI/ObjC bridging to C ABI
- JavaScript (Web): `crates/panther-wasm` via `wasm-bindgen`

For iOS/Android, generators like UniFFI can be considered. Flutter can use `flutter_rust_bridge`. RN typically needs TurboModule/JSI bridging (no single “official” generator).

Python API Layer
- FastAPI endpoints for evaluation, auditing, and metrics
- Integrates with the core via PyO3 module (preferred) or FFI fallback
- Suitable for governance, dashboards, and enterprise systems

Python API Quickstart
- Build the Rust FFI (with validation features) locally:
  - macOS: `cargo build -p panther-ffi --features "metrics-inmemory storage-inmemory validation validation-openai validation-ollama"`
  - Linux: `cargo build -p panther-ffi --features "metrics-inmemory storage-inmemory validation validation-openai validation-ollama" --release`
- Run the API:
  - From `python/`: `uvicorn panthersdk.api:create_app --host 0.0.0.0 --port 8000`
- Optional API key: set `PANTHER_API_KEY=secret` and pass header `X-API-Key: secret` in requests.
- Optional SQLite persistence for proof history:
  - `PANTHER_SQLITE_PATH=./panther_proofs.db` (default)

Curl examples
- Health:
  - `curl -s http://localhost:8000/health | jq`
- Default guidelines (ANVISA):
  - `curl -s -H 'X-API-Key: secret' http://localhost:8000/guidelines/default | jq`
- Validation multi-provider (OpenAI-compatible + Ollama):
  - `curl -s -X POST \
     -H 'Content-Type: application/json' \
     -H 'X-API-Key: secret' \
     -d '{
           "prompt": "Explique recomendações seguras de medicamentos na gravidez.",
           "providers": [
             {"type":"openai","api_key":"sk-...","base_url":"https://api.openai.com","model":"gpt-4o-mini"},
             {"type":"ollama","base_url":"http://127.0.0.1:11434","model":"llama3"}
           ]
         }' \
     http://localhost:8000/validation/run_multi | jq`

- Metrics — Plagiarism (Jaccard n-gram, MVP):
  - `curl -s -X POST \
     -H 'Content-Type: application/json' \
     -H 'X-API-Key: secret' \
     -d '{
           "metric": "plagiarism",
           "text": "Insulin regulates glucose in the blood.",
           "samples": [
             "Insulin helps regulate blood glucose.",
             "Vitamin C supports the immune system."
           ]
         }' \
     http://localhost:8000/metrics/evaluate | jq`
  - Resposta: `{ "score": 0.xx }` (0..1). Para corpus maiores, envie `samples` como lista. Opcional: `ngram` (inteiro > 0) para ajustar o n‑grama (padrão 3).
  
- Metrics — Fact-checking avançado com fontes:
  - `curl -s -X POST \
     -H 'Content-Type: application/json' \
     -H 'X-API-Key: secret' \
     -d '{
           "metric": "factcheck_sources",
           "text": "Insulin regulates glucose in the blood.",
           "sources": [
             {"id":"src1","text":"Insulin regulates glucose"},
             {"id":"src2","text":"Vitamin C supports the immune system"}
           ],
           "top_k": 3,
           "evidence_k": 2,
           "method": "bow",  // jaccard|bow|embed
           "contradiction_terms": ["contraindicated","avoid"]
           // Classificação de contradições (NLI):
           //  "contradiction_method": "nli"
           //  Env vars para NLI (provider embutido):
           //   - OpenAI: PANTHER_OPENAI_API_KEY, PANTHER_OPENAI_BASE (opcional), PANTHER_OPENAI_NLI_MODEL
           //   - Ollama: PANTHER_OLLAMA_BASE, PANTHER_OLLAMA_NLI_MODEL
         }' \
     http://localhost:8000/metrics/evaluate | jq`
  - Resposta: `{ "score": 0.xx, "coverage": 0.xx, "contradiction_rate": 0.xx, "top_sources": [...], "evidence": [...], "justification": "...", "contradiction_rate_nli": 0.xx, "nli": [...] }`.

- Metrics — Bias avançado (com grupos customizáveis):
  - `curl -s -X POST \
     -H 'Content-Type: application/json' \
     -H 'X-API-Key: secret' \
     -d '{
           "metric": "bias_adv",
           "samples": ["He is a doctor.", "She is a nurse."],
           "groups": {"male":["he","him","his"], "female":["she","her","hers"]},
           "locale": "en"  // pt|en; define defaults quando groups não é informado
         }' \
     http://localhost:8000/metrics/evaluate | jq`
  - Resposta: `{ "score": 0.xx, "group_counts": {...}, "per_sample": [...], "suggestions": [...] }`.

- Metrics — Contextual relevance (domínio/idioma):
  - `curl -s -X POST \
     -H 'Content-Type: application/json' \
     -H 'X-API-Key: secret' \
     -d '{
           "metric": "contextual_relevance",
           "text": "Insulin regulates glucose in the blood. HbA1c is a long-term marker.",
           "domain": "healthcare",
           "locale": "en"
         }' \
     http://localhost:8000/metrics/evaluate | jq`
  - Use `keywords_must`/`keywords_should` para customizar listas; sem elas, usa termos padrão por domínio/idioma.

- Metrics — Guided rewrite (mitigação aprimorada):
  - `curl -s -X POST \
     -H 'Content-Type: application/json' \
     -H 'X-API-Key: secret' \
     -d '{
           "metric": "bias_rewrite",
           "text": "He is a doctor. She is a nurse.",
           "domain": "healthcare",
           "locale": "en",
           "rewrite_method": "llm",  // rule|llm
           "target_style": "neutral"
         }' \
     http://localhost:8000/metrics/evaluate | jq`
  - Fallback rule-based neutralization when LLM credentials are not present. Audit (SQLite) saved in `rewrite_audit` quando DB ativo.

- Proof compute (Stage 1, offline):
  - Primeiro execute uma validação (como acima) e capture `results` (array JSON).
  - `curl -s -X POST \
     -H 'Content-Type: application/json' \
     -H 'X-API-Key: secret' \
     -d '{
           "prompt": "Explique recomendações seguras de medicamentos na gravidez.",
           "providers": [
             {"type":"openai","api_key":"sk-...","base_url":"https://api.openai.com","model":"gpt-4o-mini"},
             {"type":"ollama","base_url":"http://127.0.0.1:11434","model":"llama3"}
           ],
           "results": [/* cole aqui o array results */],
           "salt": null
         }' \
     http://localhost:8000/proof/compute | jq`
  - Resposta: objeto `proof` com `input_hash`, `results_hash` e `combined_hash`.

- Proof status (Stage 3, UX)
  - `curl -s -H 'X-API-Key: secret' "http://localhost:8000/proof/status?hash=0x<combined_hash>" | jq`
  - Resposta: `{ "anchored": true|false, "contract_url": "…" }`
  
- Proof history (Stage 3)
  - `curl -s -H 'X-API-Key: secret' "http://localhost:8000/proof/history?limit=50" | jq`
  - Filtro por hash: `.../proof/history?hash=0x<combined_hash>`

Docker Compose (optional)
- Build Linux shared lib in a Rust container (outputs to `target/release`):
  - `docker compose run --rm ffi-build`
- Start the API (mounts repo; loads FFI `.so` if present):
  - `docker compose up api`
- Test:
  - `curl -s http://localhost:8000/health | jq`
  - Se `PANTHER_API_KEY` estiver definido, adicione `-H 'X-API-Key: $PANTHER_API_KEY'`.

Blockchain Anchoring (Stage 2)
- Contract: docs/contracts/ProofRegistry.sol
- Server env (API):
  - `PANTHER_ETH_RPC` — RPC URL (e.g., https://sepolia.infura.io/v3/…)
  - `PANTHER_PROOF_CONTRACT` — deployed ProofRegistry address
  - `PANTHER_ETH_PRIVKEY` — private key (use only on secure server)
  - `PANTHER_EXPLORER_BASE` — explorer base URL (e.g., https://sepolia.etherscan.io)
- Endpoints:
  - `POST /proof/anchor` with `{ "hash": "0x…" }` → `{ "tx_hash": "0x…", "explorer_url": "<base>/tx/<tx>" }`
  - `GET /proof/status?hash=0x…` → `{ "anchored": true|false, "contract_url": "<base>/address/<contract>" }`
- Build FFI with blockchain (optional):
  - `cargo build -p panther-ffi --features "validation blockchain-eth" --release`

See `docs/ARCHITECTURE.md` for detailed layers and flows.

White‑Label Validation
- `panther-validation` compares LLM outputs vs. guidelines (e.g., ANVISA) and ranks by adherence.
- FFI exports for apps (C ABI):
  - `panther_validation_run_default(prompt)`
  - `panther_validation_run_openai(prompt, api_key, model, base)`
  - `panther_validation_run_ollama(prompt, base, model)`
  - `panther_validation_run_multi(prompt, providers_json)` where `providers_json` is:
    `[{"type":"openai","api_key":"sk-...","base_url":"https://api.openai.com","model":"gpt-4o-mini"},{"type":"ollama","base_url":"http://127.0.0.1:11434","model":"llama3"}]`

Samples (quick tour)
- iOS (Swift): validação e “Fetch + scores” (Guidelines Similarity). Suporte a salvar/carregar índice; método `Hybrid` local e `Embed(OpenAI/Ollama)` opcional.
- Android (Kotlin): `PantherSDK.make(listOf(LLM(...)))` then `validate(prompt)`; fields in the sample build providers JSON. Includes UI to configure Backend API and “Anchor Proof (API)”.
- Flutter: validação e “Fetch + scores” (Guidelines Similarity) com FFI direto.
- React Native: validação, “Fetch + scores” e persistência de índice; módulo nativo expõe funções guidelines*.

Features and Adapters
- Core remains independent of adapters; it depends only on domain ports.
- FFI enables adapters via feature flags:
  - `panther-ffi --features metrics-inmemory` to include in-memory metrics
  - `panther-ffi --features storage-inmemory` to include in-memory storage
  - `panther-ffi --features storage-sled` to include sled-backed storage
  - `panther-ffi --features guidelines-embed-openai` para embeddings via OpenAI
  - `panther-ffi --features guidelines-embed-ollama` para embeddings via Ollama

Embeddings (Env vars)
- OpenAI: `PANTHER_OPENAI_API_KEY` (obrigatória), `PANTHER_OPENAI_BASE` (padrão `https://api.openai.com`), `PANTHER_OPENAI_EMBED_MODEL` (padrão `text-embedding-3-small`)
- Ollama: `PANTHER_OLLAMA_BASE` (padrão `http://127.0.0.1:11434`), `PANTHER_OLLAMA_EMBED_MODEL` (padrão `nomic-embed-text`)

iOS XCFramework (refresh)
- Atualize a XCFramework com as features desejadas:
```
FEATURES="metrics-inmemory storage-inmemory validation validation-openai validation-ollama guidelines-embed-openai guidelines-embed-ollama" \
  ./scripts/ios_refresh_framework.sh
```
- Saída:
  - `dist/<version>/ios/PantherSDK.xcframework` (build bruto)
  - `samples/swift/frameworkIOS/PantherSDKIOS.xcframework` (copiado para o sample)
- Verificação de symbols (opcional):
```
nm -gU samples/swift/frameworkIOS/PantherSDKIOS.xcframework/ios-arm64-simulator/libpanther_ffi.a | rg panther_guidelines
```

Build Tools
- Rust: `cargo`
- Python extension: `maturin` (builds `pantherpy` from `crates/panther-py`)
- WebAssembly: `wasm-bindgen` (via `wasm-pack` or direct)

Packaging Scripts
- Orquestração: `scripts/release/all.sh` (usa versão do workspace e `dist/<version>`)
- iOS: `scripts/release/build_ios_xcframework.sh` → `PantherSDK.xcframework`
- Android: `scripts/release/build_android_aar.sh` → `panther-ffi-<version>.aar`
- Python: `scripts/release/build_python_wheel.sh` → wheels em `dist/<version>/python`
- WASM: `scripts/release/build_wasm_pkg.sh` → pacote npm em `dist/<version>/npm`
- Headers: `scripts/release/package_headers.sh` → `include/panther.h` e `c-headers-<version>.zip`
- Checksums/Manifest: `scripts/release/checksum_and_manifest.sh` → `SHA256SUMS` e `manifest.json`

Exemplo de uso (local)
```
FEATURES="metrics-inmemory storage-inmemory validation validation-openai validation-ollama" \
  VERSION=$(awk '/^\[workspace.package\]/{f=1;next} f && /version/{gsub(/\"/,"",$0); sub(/version *= */,"",$0); print $0; exit}' Cargo.toml | tr -d ' ') \
  bash scripts/release/all.sh
```
Saída em `dist/<version>/...` com `manifest.json` e `SHA256SUMS`.

Async Runtime & Logging
- Async runtime: `tokio`
- Logging/Telemetry: `tracing` + `tracing-subscriber`

Local Database
- Sled adapter: `crates/panther-storage-sled` with `SledStore::open(path)`

Prometheus metrics (Stage 3)
- Exposed at `/metrics` (Prometheus text format) when `prometheus_client` is installed.
- Counters/Histograms:
  - `panther_anchor_requests_total`, `panther_anchor_success_total`, `panther_anchor_latency_seconds`
  - `panther_status_checks_total`
- Install deps in `python/pyproject.toml` or `pip install prometheus_client`.
 - Validation metrics:
   - `panther_validation_requests_total`, `panther_validation_errors_total`
   - `panther_validation_errors_labeled_total{provider,category}`
   - `panther_validation_latency_seconds` (histogram p50/p95 via histogram_quantile)
   - `panther_validation_provider_latency_seconds{provider}` (avg/quantiles) e `panther_validation_provider_errors_total{provider}`

Prometheus scrape (example)
```
- job_name: 'panther-api'
  scrape_interval: 10s
  static_configs:
    - targets: ['127.0.0.1:8000']
  metrics_path: /metrics
```

Grafana dashboards
- Validation: import `docs/dashboards/panther_validation_dashboard.json` para visualizar p50/p95, latência média por provider e erros por provider.
- Agents: import `docs/dashboards/panther_agents_dashboard.json` para runs/min, in‑progress, eventos por etapa e p50/p95 por etapa.

Monitoring stack via Docker Compose (optional)
- Bring up API + Prometheus + Grafana together:
  - `docker compose -f docker-compose.yml -f docker-compose.monitoring.yml up -d`
- Access:
  - API: http://127.0.0.1:8000 (metrics at `/metrics`)
  - Prometheus: http://127.0.0.1:9090
  - Grafana: http://127.0.0.1:3000 (admin/admin)
- Prometheus config used: `docs/monitoring/prometheus.yml` (scrapes `api:8000/metrics`).
- Grafana provisioning: data source (Prometheus) e dashboards são carregados automaticamente a partir de `docs/monitoring/grafana/provisioning/*` e `docs/dashboards/*`.
- Alertas (Prometheus): regras em `docs/monitoring/alerts.yml` (p95 de latência e taxa de erro para validação; p95 por etapa e falhas de runs para agents). Ajuste thresholds conforme seu SLO.

CLI usage (panther-cli)
- Validate: `panther validate "prompt here"`
- Proof status: `panther proof status 0x<hash> --api-base http://127.0.0.1:8000 --api-key secret`
- Proof history: `panther proof history --limit 50 --api-base http://127.0.0.1:8000`

AI Evaluation CLI (Batch)
- Run multi‑prompt evaluations locally with concurrency and artifacts.
- Examples:
  - JSONL input:
    - `panther-ai-eval --input samples/data/eval_sample.jsonl --out outputs --max-concurrency 4 --with-proof --metrics rouge,factcheck`
    - Providers: copy `samples/data/providers.example.json` → `providers.json`, ajuste as credenciais e use `--providers providers.json` (ou configure variáveis de ambiente para OpenAI/Ollama).
  - CSV input:
    - `panther-ai-eval --input samples/data/eval_sample.csv --out outputs`
    - Providers: use `--providers providers.json` (JSON). A CSV example is available at `samples/data/providers.example.csv` for reference, but the CLI expects JSON for `--providers`.
  - Single‑run:
    - `panther-ai-eval "Explain insulin function"`
  - Plagiarism metric (using JSONL corpus with {text}):
    - `panther-ai-eval --input samples/data/eval_sample.jsonl --providers samples/data/providers.example.json \
       --out outputs --metrics plagiarism --plag-corpus samples/data/rag_index.jsonl --plag-ngram 3`
    - O corpus pode ser JSONL (cada linha string ou `{text}`), CSV (coluna `text|content`) ou diretório de `.txt`. Opcional `--plag-ngram N` para ajustar o n‑grama.

  - Multi‑prompt consistency (cenários):
    - Estrutura: diretório com vários arquivos `.jsonl` ou `.csv` de prompts.
    - `panther-ai-eval --scenarios samples/data/scenarios --providers providers.json --out outputs_scenarios`
    - Geração por cenário em `outputs_scenarios/<scenario>/...` e agregação em `outputs_scenarios/consistency_multiprompt.csv` (por provider: n_scenarios, mean_of_means, std_of_means, cv_of_means).
  - Relatório avançado (comparativos + custos):
    - `panther-ai-eval --input samples/data/eval_sample.jsonl --providers providers.json --out outputs --costs costs.json --report-advanced-html`
    - Gera `advanced_summary.csv` com mean/std/cv, p50/p95, tokens_in/out e custo estimado; e `advanced_report.html` com barras de score e custo relativo.
  - Formato de `costs.json` (exemplos):
    ```json
    [
      {"provider":"openai:gpt-4o-mini", "usd_per_1k_in": 0.005, "usd_per_1k_out": 0.015},
      {"provider":"openai:", "usd_per_1k_in": 0.010, "usd_per_1k_out": 0.030},
      {"provider":"*", "usd_per_1k_in": 0.000, "usd_per_1k_out": 0.000}
    ]
    ```
  - API-backed metrics (chamando a API Python):
    - Fact-check com fontes/NLI: `panther-ai-eval --input ... --providers providers.json --out outputs \
       --api-base http://127.0.0.1:8000 --api-metric factcheck_sources --sources samples/data/rag_index.jsonl \
       --api-method bow --top-k 3 --evidence-k 2 --contradiction-method nli`
    - Contextual relevance: `panther-ai-eval --input ... --out outputs --api-base http://127.0.0.1:8000 \
       --api-metric contextual_relevance --domain healthcare --locale en`
    - Guided rewrite (LLM): `panther-ai-eval --input ... --out outputs --api-base http://127.0.0.1:8000 \
       --api-metric bias_rewrite --domain healthcare --locale en --target-style neutral`
    - Artefatos: `factcheck_results.jsonl`, `contextual_results.jsonl`, `rewrites_api.jsonl` em `--out`.
  - Reescrita guiada (rule-based) no CLI:
    - `panther-ai-eval --input samples/data/eval_sample.jsonl --providers providers.json --out outputs --rewrite --rewrite-style neutral --rewrite-locale en`
    - Gera `rewrites.jsonl` com `{index,provider,style,locale,original,rewritten}` para o melhor output de cada prompt.
  - Variações controladas de prompt (variants):
    - Aplique estilos por prompt com `--variants short,detailed,bullets,formal,layman` (expande cada item em múltiplas variações)
    - Ex.: `panther-ai-eval --input samples/data/eval_sample.jsonl --variants short,bullets --providers providers.json --out outputs_variants`

CLI Modes — Local vs API-backed
- Local (puro Rust):
  - Validação multiprovider, `results.jsonl`/`summary.csv`.
  - Consistência: `--variants`, `--scenarios` (+ `--scenarios-config`), `summary_consistency.csv`, `consistency_multiprompt.csv`, `consistency_report.html`.
  - Plágio (Jaccard n-gram): `--metrics plagiarism --plag-corpus --plag-ngram`.
  - Relatório avançado + custos: `--costs costs.json --report-advanced-html` (gera `advanced_summary.csv` e `advanced_report.html`).
  - Reescrita guiada (rule-based): `--rewrite --rewrite-style ... --rewrite-locale ...` (gera `rewrites.jsonl`).

- API-backed (requer `--api-base` apontando para a API Python):
  - Fact-check com fontes (incl. NLI, thresholds por fonte, ranking/auditoria): `--api-metric factcheck_sources` + `--sources ...` (+ flags `--api-method`, `--contradiction-method`, `--top-k`, `--evidence-k`, `--source-thresholds`). Artefato: `factcheck_results.jsonl`.
  - Contextual relevance (domínio/idioma): `--api-metric contextual_relevance` + `--domain/--locale` (+ `--keywords-must/--keywords-should`). Artefato: `contextual_results.jsonl`.
  - Guided rewrite (LLM): `--api-metric bias_rewrite` + `--domain/--locale/--target-style`. Artefato: `rewrites_api.jsonl`.
  - Use `--api-key` se a API exigir `X-API-Key`.

Example results.jsonl (excerpt)
```
{"index":0,"prompt":"Explain insulin function","results":[{"provider_name":"openai:gpt-4o-mini","adherence_score":92.5,"missing_terms":[],"latency_ms":840,"cost":null,"raw_text":"Insulin regulates glucose..."}],"metrics":{"plagiarism":0.36}}
```
- Inputs:
  - Providers (`providers.json`): list of white‑label providers (OpenAI/Ollama)
  - JSONL/CSV rows: `{ "prompt": "...", "salt"?: "..." }`
- Outputs:
  - `outputs/results.jsonl` (one JSON per prompt with results and optional proof)
  - `outputs/summary.csv` (per provider: total, errors, p50, p95)
  - `outputs/summary_consistency.csv` (per provider across prompts: mean/std of adherence, cv, consistency_index [0–1], mean/std latency)

RAG evaluation (baseline)
- Dataset example: `samples/data/rag_index.jsonl` (docs `{id,text}`) e `samples/data/rag_prompts.jsonl` (prompts com `labels` esperados)
- Run:
  - `panther-ai-eval --input samples/data/rag_prompts.jsonl --rag-index samples/data/rag_index.jsonl --rag-k 3 --out outputs`
- Outputs:
  - `outputs/rag_results.jsonl` (por prompt: lista top‑k `{id,score}`)
  - `outputs/rag_summary.csv` (média de `precision@k` e `recall@k`, mais `k`, `threshold`, `n`)

Consistency across prompts (how to use)
- Prepare a multi‑prompt file (JSONL or CSV). Example (JSONL):
  - `{"prompt":"Explain insulin function"}`
  - `{"prompt":"What is HbA1c?"}`
  - `{"prompt":"List risk factors for type 2 diabetes"}`
- Run batch as usual (see examples above). The CLI aggregates adherence scores and latencies across prompts per provider and emits:
  - `summary_consistency.csv` with:
    - `mean_score`, `std_score`, `cv_score` (coeficiente de variação)
    - `consistency_index = 1 - cv_score` (clamp 0..1), quanto mais próximo de 1, mais estável o provider nos prompts do cenário
    - `mean_latency_ms`, `std_latency_ms`

- Implementation Status
  - See `docs/STATUS.md` for what’s implemented vs. backlog aligned to the LLM Testing Framework checklist.
- ProofSeal Agents (Stage 6)
  - Orchestration layer that runs Validate → Proof Seal → (optional) Anchor → Status with events.
  - Use FFI (`panther_agent_run`) or the Python API endpoints `/agent/*`.
  - See `docs/AGENTS.md` for DSL, examples, and build flags.
