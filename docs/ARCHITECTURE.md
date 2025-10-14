Hexagonal Architecture (Clean Architecture)

Overview
- Rust core exposed via ports (traits) and use cases
- Concrete adapters for LLM providers, storage, networking, and telemetry
- Stable C FFI for mobile and UI integration
- Python layer for API/orchestration, auditing, and enterprise metrics

Layers
- Domain (`panther-domain`)
  - Entities: `Prompt`, `Completion`, `ModelSpec`, `TraceEvent`
  - Errors and result types
  - Ports (traits): `LlmProvider` (sync), `LlmProviderAsync` (async), `TelemetrySink`
- Application (`panther-core`)
  - Use cases: generation, evaluation, policies, model routing
  - Orchestrates `LlmProvider` and `TelemetrySink`
- Adapters (`panther-providers`, `panther-observability`)
  - Providers: OpenAI, Anthropic, Local (Ollama), etc.
  - Observability: `tracing` with optional OpenTelemetry/OTLP export
- Metrics/Storage Adapters (`panther-metrics`, `panther-storage`)
  - In-memory implementations for development
  - Future: Prometheus (metrics), Redis/Postgres (storage)
- Interface/Infra (`panther-ffi`)
  - Stable C ABI + `cbindgen` header generation
  - Engine initialization, sync calls (async later)
- API (Python)
  - FastAPI for auditing, metrics, offline/server evaluation
  - Enterprise integrations (auth, storage, data lake, SIEM)

Flows
- On-device: App -> FFI -> `panther-core` -> provider -> trace
- Server-side: Python receives metrics/traces, performs evaluation/policies

Bindings
- iOS/Swift: wrapper using `panther.h` (C ABI)
- Android/Kotlin: JNI over C ABI
- Flutter/Dart: `dart:ffi` over C ABI
- React Native: TurboModule/JSI with C++ wrappers calling C ABI

Initial Decisions
- Stable C ABI is the contract for all clients
- `tracing` as baseline; optional OTLP integration
- External providers are pluggable via features

Validation
- Crate `panther-validation` loads guideline sets (e.g., ANVISA) and computes adherence by searching expected terms in provider outputs.
- FFI surface for validation:
  - `panther_validation_run_default(prompt)`
  - `panther_validation_run_openai(prompt, api_key, model, base)`
  - `panther_validation_run_ollama(prompt, base, model)`
  - `panther_validation_run_multi(prompt, providers_json)` to support white‑label provider lists.
  - Stage 1 (proofs offline):
    - `panther_validation_run_multi_with_proof(prompt, providers_json)` → `{ results, proof }`
    - `panther_proof_compute(prompt, providers_json, guidelines_json, results_json, salt)` → `proof`
    - `panther_proof_verify_local(prompt, providers_json, guidelines_json, results_json, salt, proof_json)` → `i32` (1 válido)
  - Stage 2 (blockchain, opcional – feature `blockchain-eth`):
    - `panther_proof_anchor_eth(hash_hex, rpc_url, contract_addr, priv_key)` → `{ tx_hash }`
    - `panther_proof_check_eth(hash_hex, rpc_url, contract_addr)` → `{ anchored: bool }`

White‑Label Providers
- JSON schema for `providers_json` (passed to `run_multi`):
  ```json
  [
    { "type": "openai", "api_key": "sk-...", "base_url": "https://api.openai.com", "model": "gpt-4o-mini" },
    { "type": "ollama", "base_url": "http://127.0.0.1:11434", "model": "llama3" }
  ]
  ```
- OpenAI‑compatible endpoints: use `type = "openai"`, set `base_url`, `model`, and `api_key`.
- Ollama: use `type = "ollama"`, set `base_url` and `model`.

Validation Flow (Runtime)
- App/UI collects provider configs (URL/Model/API Key) and the prompt.
- App calls the FFI once (`run_multi`) with `prompt` + `providers_json`.
- FFI constructs providers from the JSON and spins a small Tokio runtime.
- `LLMValidator::validate` calls each provider in parallel and measures:
  - latency_ms
  - adherence_score = 100% − missing_terms_penalty (based on guidelines)
- FFI returns a JSON array of `ValidationResult` sorted by score.
- App formats the results (e.g., `provider – 92.5% – 860 ms`).

Proofs & Blockchain (Auditoria)
- Stage 1 (Offline):
  - O SDK calcula um `proof` com hashes SHA3‑512 sobre entradas (prompt, providers, guidelines, salt) e saídas (results), gera `combined_hash` e permite verificação local.
  - Usos: auditoria local, integridade de relatórios, trilha reproduzível.
- Stage 2 (On‑chain):
  - Contrato `ProofRegistry` (docs/contracts/ProofRegistry.sol) ancora apenas o hash (bytes32) no blockchain (ex.: testnets Ethereum/Polygon).
  - API Python expõe `/proof/anchor` e `/proof/status`; chaves privadas ficam somente no backend.

Platform Facades (Samples)
- iOS (Swift): `PantherSDK` facade wraps the FFI and exposes:
  - `PantherSDK.make(llms: [LLM]).validate(prompt:)`
  - LLM: `{ type: openai|ollama, baseURL, model, apiKey? }`
- Android (Kotlin): `PantherSDK.make(listOf(LLM(...))).validate(prompt)`
- Flutter: UI builds providers JSON and calls `validateMulti(prompt, providersJson)` via Dart FFI.
- React Native: JS module calls `validateMulti(prompt, providersJson)` (native bridge passes it to FFI).

Runtime Metrics (Core)
- `panther-core` records per‑request counters/histograms and persists to storage if configured:
  - `panther.generate.calls`
  - `panther.latency_ms`
  - `panther.tokens.input|output|total` (naive whitespace token count)
- Metrics can be queried from apps via FFI helpers (e.g., `panther_storage_list_metrics`).

Build/Features
- Enable validation features in the FFI crate to include providers:
  - `validation` (core glue)
  - `validation-openai` and/or `validation-ollama`
  - Async providers (Stage 4): enable `panther-providers/openai-async` e/ou `panther-providers/ollama-async` e utilize `LlmProviderAsync` no core/validador.
- Samples’ build scripts include these features when rebuilding the static/shared library.

Error Handling
- FFI returns JSON on success (array of `ValidationResult`) or an error JSON object:
  - `{ "error": "no providers configured" }`
  - `{ "error": "runtime init failed" }`
  - `{ "error": "... upstream provider error ..." }`
- App UIs should detect error objects and render a concise notice.

ABI/Versioning
- The C ABI is stable and generated with `cbindgen`. Keep `include` list restricted to exported symbols and add version stamps as needed.
- Consider adding an explicit `panther_abi_version()` in future changes.
