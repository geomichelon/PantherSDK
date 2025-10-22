# TODO — SDK tasks to unlock full iOS sample

Scope: this file only lists what is still required on the SDK side (Rust/FFI/backends) to fully support features we want to expose in the iOS sample. Items marked below are pending SDK work — once delivered, the sample UI can be wired to them.

Current SDK coverage (already available for the iOS sample):
- Validation providers: OpenAI, Ollama, Default (env); modes: single/multi/custom/with_proof
- Proofs: `compute_proof` (offline) and `verify_proof_local`
- Metrics helpers: `panther_token_count`, `panther_calculate_cost`
- Bias basics: `panther_bias_detect`
- Storage: in‑memory and sled (KV); not yet used by the iOS sample UI
- Prometheus (partial): `panther_prometheus_scrape` behind feature — needs finalization and sample integration

Everything below is SDK PENDING work.

## [SDK PENDING] Providers (Anthropic)
- [ ] panther-providers: add Anthropic provider
  - [ ] Blocking (reqwest) and async variants with timeouts/retries
  - [ ] Config: `base_url`, `api_key`, `model`
- [ ] Features/flags
  - [ ] `validation-anthropic` (sync) and `validation-anthropic-async`
- [ ] FFI wiring (crates/panther-ffi)
  - [ ] Parse `type:"anthropic"` in `providersJson` for `run_multi`/`run_custom`/`*_with_proof`
  - [ ] Gate on the above features

## [SDK PENDING] Guidelines Ingestion + Vector Similarity
- [ ] Providers for guidelines (S3/GCS/HTTP with auth)
  - [ ] S3 (AWS SigV4) and GCS (service account / token)
  - [ ] HTTP with headers/bearer
- [ ] Similarity scoring
  - [ ] Embeddings + ANN (e.g., HNSW/FAISS or crate alternative)
  - [ ] Hybrid score (terms + vector) in `ValidationResult` (add `vector_score`/`hybrid_score`)
- [ ] FFI
  - [ ] Add `panther_guidelines_fetch(provider_config_json)` OR extend existing validation APIs to accept `guidelines_url + headers`

## [SDK PENDING] Tokenization
- [ ] Trait `Tokenizer` in panther-domain
  - [ ] `WhitespaceTokenizer` (default) and `TiktokenTokenizer` (optional)
- [ ] Integrations
  - [ ] FFI: `panther_set_tokenizer(name)` and/or `panther_token_count_with(tokenizer, text)`
  - [ ] Cost calculators accept tokenizer selection

## [SDK PENDING] Storage/Analytics (SQL)
- [ ] New crate: `panther-storage-sqlite` (rusqlite)
  - [ ] Schema for validations/metrics history
  - [ ] Queries for p50/p95/p99 by provider/model/time range
- [ ] FFI
  - [ ] `panther_sql_open(path)` / `panther_sql_export(path)`
  - [ ] `panther_sql_query_*` for summaries and history

## [SDK PENDING] Metrics/Observability
- [ ] Exporter (pull) in Rust
  - [ ] New crate `panther-metrics-prom` with `/metrics`
  - [ ] Plug p50/p95/p99 summaries
- [ ] Cross‑FFI traces
  - [ ] Propagate `traceparent` (or similar) across FFI boundaries
- [ ] FFI
  - [ ] Finalize `panther_prometheus_scrape` (feature `metrics-prometheus`) and sample usage docs

## [SDK PENDING] FFI/Binaries + Tests
- [ ] Automate bindings (investigate UniFFI/flutter_rust_bridge for Flutter/RN)
- [ ] Cross‑FFI smoke tests
  - [ ] CI builds iOS/Android/Flutter/RN and invokes 1–2 functions (validate/token/cost)
- [ ] cbindgen
  - [ ] Add `[defines]` for new features to silence WARNs

## [SDK PENDING] Python API
- [ ] Policy evaluation workflows (endpoints + orchestration)
- [ ] JWT + audit logging
- [ ] Wheels + publishing for `panther-py` (maturin) and pipeline

## [SDK PENDING] Blockchain
- [ ] Automate contract deploy (hardhat/foundry) + return explorer URLs
- [ ] Document environment variables used by `/proof/anchor` and `/proof/status`

## [SDK PENDING] Build/Release
- [ ] CI/CD: GitHub Actions (macos-14) for XCFramework
  - [ ] Use `scripts/release/build_ios_xcframework.sh`
  - [ ] Package as `PantherSDKIOS-<version>.zip` and upload to Releases
- [ ] Keep `scripts/ios_refresh_framework.sh` as local helper for samples

## iOS sample follow‑ups (to implement after SDK work lands)
- [ ] Add “Anthropic” preset in UI (enable when provider exists)
- [ ] Add S3/GCS auth UI for guideline ingestion (call new FFI)
- [ ] Tokenizer selector (Whitespace/Tiktoken) for tokens/costs
- [ ] Analytics/History tab (SQL p50/p95/p99)
- [ ] Prometheus view using `panther_prometheus_scrape`
