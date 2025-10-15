PantherSDK (Rust core + Python API)

PantherSDK is a cross-platform SDK for evaluation, analysis, and integration of generative AI (LLMs). It uses a Rust core and a Python API layer, following Hexagonal (Clean) Architecture.

Goals
- Unified base for LLM evaluation, analysis, and integration
- Usable from iOS, Android, Flutter, and React Native
- Automatic bindings across platforms via stable FFI or generators (PyO3 for Python, wasm-bindgen for JS, UniFFI optional for Swift/Kotlin)
- Auditing, metrics, and enterprise integrations

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
- iOS (Swift): `PantherSDK.make(llms:)` then `validate(prompt:)`; the UI lets you input URL/key/model for any provider.
- Android (Kotlin): `PantherSDK.make(listOf(LLM(...)))` then `validate(prompt)`; fields in the sample build providers JSON. Includes UI to configure Backend API and “Anchor Proof (API)”.
- Flutter: builds providers JSON and calls `validateMultiWithProof(prompt, providersJson)` via Dart FFI. Includes “Anchor Proof (API)”.
- React Native: native module exposes `validateMultiWithProof(...)` and helper `anchorProof(...)`. Example screen: `samples/react_native/AppSample.tsx`.

Features and Adapters
- Core remains independent of adapters; it depends only on domain ports.
- FFI enables adapters via feature flags:
  - `panther-ffi --features metrics-inmemory` to include in-memory metrics
  - `panther-ffi --features storage-inmemory` to include in-memory storage
  - `panther-ffi --features storage-sled` to include sled-backed storage

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

Grafana dashboard
- Import `docs/dashboards/panther_validation_dashboard.json` em Grafana para visualizar p50/p95, latência média por provider e erros por provider.

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
- Inputs:
  - Providers (`providers.json`): list of white‑label providers (OpenAI/Ollama)
  - JSONL/CSV rows: `{ "prompt": "...", "salt"?: "..." }`
- Outputs:
  - `outputs/results.jsonl` (one JSON per prompt with results and optional proof)
  - `outputs/summary.csv` (per provider: total, errors, p50, p95)

- Implementation Status
  - See `docs/STATUS.md` for what’s implemented vs. backlog aligned to the LLM Testing Framework checklist.
- ProofSeal Agents (Stage 6)
  - Orchestration layer that runs Validate → Proof Seal → (optional) Anchor → Status with events.
  - Use FFI (`panther_agent_run`) or the Python API endpoints `/agent/*`.
  - See `docs/AGENTS.md` for DSL, examples, and build flags.
