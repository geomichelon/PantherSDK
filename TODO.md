# TODO — Single Source of Truth (SDK + Samples)

Este arquivo é a checklist canônica de trabalho pendente e concluído. Mantenha sincronizado com ROADMAP.md e docs/FAQ.md.

Estados
- [x] Done — entregue/funcionando
- [~] In‑progress — parcialmente entregue
- [ ] Next — fazer em seguida
- [b] Backlog — planejado

Cross‑cutting
- [x] Guidelines Similarity FFI (ingest JSON; BOW/Jaccard/Hybrid)
- [x] Documentar no Help/README que BOW/Jaccard/Hybrid são locais/offline; Embeddings requerem endpoints
- [x] Embeddings (OpenAI/Ollama) para Similarity via FFI
- [x] Persistência de diretrizes (save/load via KV)
- [ ] ANN opcional para embeddings (HNSW/IVF) + cache de índices
- [ ] Tokenizer abstrato (Whitespace/Tiktoken) + seleção via FFI
- [ ] cbindgen: adicionar [defines] p/ features e silenciar WARNs

SDK (Rust)
- Validation
  - [x] Provedores: OpenAI, Ollama, Anthropic (sync/async por features)
  - [x] FFI: default/openai/ollama/multi/custom + with_proof
- Proof
  - [x] `compute_proof`/`verify_proof_local`
  - [x] ETH anchor/check (feature `blockchain-eth`)
- Storage
  - [x] In‑Memory / Sled
  - [ ] SQLite (histórico/analytics; p50/p95/p99)
- Métricas
  - [x] `panther_token_count`, `panther_calculate_cost`
  - [ ] Exportador Prometheus (Rust) + `/metrics` unificado
- FFI/Headers
  - [x] Incluir guidelines* no `cbindgen.toml`
  - [ ] Testes FFI (smoke tests iOS/Android/Flutter/RN em CI)

Python API
- [x] Endpoints principais (generate/metrics/bias/validation)
- [ ] Workflows de policy evaluation
- [ ] JWT + audit logging
- [ ] Wheels + publicação (`panther-py`)

Build/Release
- [x] `ios_refresh_framework.sh` limpa xcframework antes de recriar
- [ ] CI/CD iOS (GHA macos‑14) empacotando XCFramework e subindo em Releases
- [ ] Manifest/checksums unificados por plataforma

Samples — Padrão de Navegação + Sessão (Providers | Validate | Guidelines)
- iOS (Swift)
  - [x] Abas Providers/Validate/Guidelines
  - [x] `AppSession` persistente (UserDefaults)
  - [x] Validate consome sessão; Guidelines com Fetch+scores e salvar/carregar índice
  - [ ] Cleanup final: remover estados locais remanescentes não usados em Validate
- Android (Kotlin)
  - [x] Salvar/Carregar sessão (SharedPreferences)
  - [ ] Separar UI em 3 “abas” simples (botões que alternam blocos)
- React Native
  - [x] Salvar/Carregar sessão (AsyncStorage)
  - [ ] TabBar interna (3 views: Providers/Validate/Guidelines) mantendo estados
- Flutter
  - [x] Salvar/Carregar sessão (SharedPreferences)
  - [ ] Migrar para `DefaultTabController` (3 tabs) com Providers/Validate/Guidelines

Docs
- [x] README.md atualizado (PoC, FFI, embeddings, iOS build)
- [x] ROADMAP.md atualizado (guidelines/embeddings marcados como feitos; próximos passos)
- [x] FAQ (docs/FAQ.md)
- [ ] CHANGELOG.md curto para apresentações
- [ ] Capturas de tela/GIFs das telas

Próximos (ordem sugerida)
1) Tabs em RN e Flutter (padrão unificado)
2) ANN p/ embeddings (HNSW) e cache de índice
3) SQLite (histórico/analytics) + tela “Analytics” (p50/p95)
4) Exportador Prometheus (Rust) + dashboards exemplo
5) Testes FFI em CI + empacotamento artefatos (xcframework/aar/wheel/npm)
