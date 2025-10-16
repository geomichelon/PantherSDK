PantherSDK — Roadmap e Checklist

Fonte de verdade: `panther_features.yaml` e `docs/panther_features_todo*.yaml`.
Este arquivo consolida o estado atual em formato checklist, por feature e por crate.

Visão Rápida
- [x] Core runtime (Engine, Telemetria, Métricas básicas)
- [x] FFI C estável (+ helpers de métricas/armazenamento/logs)
- [x] Validação de LLMs (guidelines ANVISA, ranking, CLI)
- [x] API Python (FastAPI) com fallbacks
- [x] Storage KV (in-memory e sled)
- [x] Samples: Swift, Kotlin, Flutter, React Native
- [x] Versão em FFI (`panther_version_string` dinâmico)
- [x] Providers assíncronos (OpenAI/Ollama — básicos com timeouts/retries)
- [ ] Anthropic (assíncrono)
- [ ] Exportador Prometheus (Rust) e agregações p50/p95 consolidadas
- [x] p50/p95 no CLI de validação
- [ ] SQL analytics (SQLite/Postgres)
- [ ] Suíte de testes cross‑FFI e automação de bindings
 - [x] RAG baseline no `panther-ai-eval` (flags `--rag-*` e artifacts)

Por Feature (do panther_features.yaml)

Core Runtime
- [x] Orquestração (`Engine`) e telemetria básica
- [x] Métricas simples (contadores/histogramas) e token count
- [x] Providers: `NullProvider` + OpenAI/Ollama (bloco síncrono)
- [x] `LlmProvider` assíncrono e adapters de produção (OpenAI/Ollama)
- [ ] Tokenizer abstrato

Storage e Dados
- [x] KV In‑Memory (`panther-storage`)
- [x] Sled (`panther-storage-sled`)
- [ ] Esquema persistente para avaliações/metrics (SQL)
- [ ] Analytics SQL (SQLite/Postgres)
- [ ] Export/import `.pantherdb`

FFI Cross‑Platform
- [x] C ABI (`panther-ffi`) com `cbindgen`
- [x] Funções: init/generate/version, validação default/openai/ollama/multi/custom
- [x] Helpers: métricas/armazenamento/logs
- [ ] Automação de bindings (UniFFI/flutter_rust_bridge)
- [ ] Testes de integração entre Swift/Kotlin/Flutter/RN

Python API
- [x] FastAPI endpoints: `generate`, `metrics/evaluate`, `metrics/history`, `bias/analyze`
- [x] PyO3 módulo (`panther-py`) com `init/generate/bleu/get_history/detect_bias`
- [ ] Workflows de policy evaluation
- [ ] JWT + audit logging
- [x] Endpoints de guidelines/validação integrados

Compliance/Guidelines
- [x] `LLMValidator` (paraleliza, mede latência, ranqueia score)
- [x] Guidelines ANVISA de exemplo
- [ ] Ingestão de guidelines (Drive/S3)
- [ ] Busca vetorial + similaridade
- [ ] `trust_index` e `bias_score` avançados

Blockchain / Verificação
- [x] Stage 1: prova offline (`compute_proof`, `verify_proof_local`)
- [x] Stage 1 FFI: `panther_validation_run_multi_with_proof`, `panther_proof_compute`, `panther_proof_verify_local`
- [x] Stage 2 base: contrato `ProofRegistry` + FFI `panther_proof_anchor_eth`/`panther_proof_check_eth`
- [x] API Python: `/proof/anchor` e `/proof/status`
- [ ] Samples: botão “Anchor proof” via API (UI)
- [ ] Automação de deploy do contrato (testnet) e link para explorer

Observabilidade/Métricas
- [x] `tracing` + sink de logs
- [ ] Exportador Prometheus (Rust) e integração ampla
- [x] Agregações p50/p95 (Prometheus/dashboards)
- [x] Métricas Prometheus no backend (Python) para proofs/agents/validação
- [ ] Correlação de spans cross‑FFI

Developer Experience
- [x] Workspace Cargo e `cbindgen.toml`
- [x] `panther-cli` para validação
- [x] Scripts de build para samples
- [ ] Empacotamento unificado (.xcframework, .aar, wheel)
- [ ] CI/CD de artefatos

Samples
- [x] iOS (Swift): métricas/logs/validação, presets de providers, cache de guidelines
- [x] Android (Kotlin/JNI): métricas/logs/validação, presets, cache de guidelines
- [x] Flutter (Dart FFI): métricas/validação
- [x] React Native (TurboModule/JSI): métricas/logs/validação
- [ ] Relatórios de compliance nas demos
- [ ] Fluxos de bias e trust em todos os samples

Por Crate/Componente

panther-domain
- [x] Entidades: `Prompt`, `Completion`, `ModelSpec`, `TraceEvent`
- [x] Portas: `LlmProvider`, `TelemetrySink`, `MetricsSink`, `KeyValueStore`
- [x] Errors: `PantherError`

panther-core
- [x] `Engine::generate` com telemetria, métricas e persistência
- [x] `generate_async` (assinatura pronta; corpo síncrono)
- [x] Providers assíncronos reais (OpenAI/Ollama) via traits

panther-providers
- [x] `NullProvider`
- [x] OpenAI/Ollama (reqwest blocking)
- [x] `reqwest` async + timeouts/retries (OpenAI/Ollama)
- [ ] Anthropic (sync/async)

panther-observability
- [x] `init_logging` e `LogSink`
- [ ] OTLP/OTel opcional

panther-metrics
- [x] `InMemoryMetrics` (contadores/histogramas)
- [ ] Exportadores/coletores

panther-storage / panther-storage-sled
- [x] In‑Memory KV / Sled KV
- [x] Round‑trips testados
- [ ] Adapters SQL

panther-ffi
- [x] FFI estável: `panther_init`, `panther_generate`, `panther_version_string`, free
- [x] Validação: default/openai/ollama/multi/custom (feature‑gated)
- [x] Helpers métricas/armazenamento/logs
- [ ] Suite de testes FFI

panther-validation
- [x] `LLMValidator`, `ProviderFactory`
- [x] Exemplo `anvisa.json`
- [ ] Similaridade vetorial

panther-cli
- [x] Comando `validate` com saida tabular
- [x] Expor p50/p95 e sumarização

panther-py
- [x] Módulo PyO3 `pantherpy`
- [x] Funções `init`, `generate`, `evaluate_bleu_py`, `get_history_py`, `detect_bias_py`
- [ ] Wheels + publicação

python/panthersdk
- [x] FastAPI: `generate`, `metrics/evaluate`, `metrics/history`, `bias/analyze`
- [ ] Autenticação/autorização, auditoria

panther-wasm
- [x] Binding wasm para `panther_init`/`panther_generate`
- [ ] Pipeline de build web + demo

panther-ai-eval
- [x] CLI de avaliação com saída e artefatos JSON
- [x] Cenários adicionais e batch (JSONL/CSV, concorrência, results.jsonl/summary.csv)
 - [x] RAG: `--rag-*` com `rag_results.jsonl/rag_summary.csv/rag_experiments.csv`

panthersdk (aggregator)
- [x] Re‑exports e domínios utilitários: `metrics`, `storage`, `bias`, `runtime`

Milestones Próximos
- Providers assíncronos + timeouts/retries (OpenAI/Ollama)
- Prometheus exporter + p50/p95
- SQL storage (SQLite) para histórico/analytics
- Suíte de testes cross‑FFI (Swift/Kotlin/Flutter/RN)
- Empacotamento unificado + CI/CD de artefatos

Como Atualizar
- Atualize status nas seções acima ao concluir entregas.
- Mantenha `panther_features.yaml` como referência e sincronize este checklist quando necessário.
