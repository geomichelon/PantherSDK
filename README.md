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

See `docs/ARCHITECTURE.md` for detailed layers and flows.

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

Async Runtime & Logging
- Async runtime: `tokio`
- Logging/Telemetry: `tracing` + `tracing-subscriber`

Local Database
- Sled adapter: `crates/panther-storage-sled` with `SledStore::open(path)`
