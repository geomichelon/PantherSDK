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
  - Ports (traits): `LlmProvider`, `TelemetrySink`
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
