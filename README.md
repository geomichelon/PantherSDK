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
  - Local methods are offline and private: BOW (Bag‑of‑Words cosine over normalized term‑frequency vectors), Jaccard (set overlap |A∩B|/|A∪B|), and Hybrid (0.6×BOW + 0.4×Jaccard). Embeddings require credentials/endpoint.
- Storage helpers (in‑memory/sled) and log capture for audit trails
- Bias and content metrics (BLEU, ROUGE‑L, accuracy, fluency, diversity, plagiarism)
- Samples for iOS (Swift), Android (Kotlin), Flutter (Dart FFI) e React Native (Android/iOS)

Demo Video
- Place your demo at `docs/media/demo.mp4` (under 100 MB) and GitHub will render it inline below.

<video src="docs/media/demo.mp4" width="720" controls muted playsinline>
  Your browser does not support the video tag. See the link below.
</video>

- Direct link: docs/media/demo.mp4

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
  - BOW/Jaccard/Hybrid run locally (offline). Use Embeddings (OpenAI/Ollama) only if you want vector similarity via external endpoints.
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
- iOS (Swift): `PantherSDK.make(llms:)` then `validate(prompt:)`; the UI lets you input URL/key/model for any provider.
- Android (Kotlin): `PantherSDK.make(listOf(LLM(...)))` then `validate(prompt)`; fields in the sample build providers JSON.
- Flutter: builds providers JSON and calls `validateMulti(prompt, providersJson)` via Dart FFI.
- React Native: native module exposes `validateMulti(prompt, providersJson)` (see sample’s `Panther.ts`).

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
