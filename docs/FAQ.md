PantherSDK — FAQ (Perguntas Frequentes)

1) O que faz a Validação Multi‑Provedor?
- Compara as respostas de diferentes LLMs (OpenAI/Ollama/Anthropic) contra diretrizes (ex.: ANVISA), calcula um score de aderência, mede latência, conta tokens e estima custo.
- FFI principais: `panther_validation_run_default|openai|ollama|multi|custom`, além de variantes `*_with_proof` para incluir a prova.

2) O que é a “Prova” (Proof) e como ancorar?
- A prova é um hash combinado que identifica de forma reprodutível o processo de validação (prompt, providers, diretrizes, resultados). Pode ser verificada localmente.
- FFI: `panther_proof_compute`, `panther_proof_verify_local`. Anchoring opcional (ETH) via `panther_proof_anchor_eth` e `panther_proof_check_eth` (ou pela API Python).

3) O que é “Guidelines Similarity” e quais métodos existem?
- Mede a similaridade entre um texto de consulta (prompt) e um conjunto de diretrizes. Útil para “encontrar a regra mais relevante”.
- Métodos:
  - `bow`: Bag‑of‑Words com cosseno (vetor normalizado por frequência de termos)
  - `jaccard`: Similaridade de conjuntos de tokens
  - `hybrid`: Combinação 0.6*bow + 0.4*jaccard
  - `embed-openai` e `embed-ollama`: Similaridade vetorial com embeddings (cosseno)
- FFI: `panther_guidelines_ingest_json`, `panther_guidelines_similarity`, `panther_guidelines_embeddings_build`.

4) Como habilitar embeddings para Similarity?
- Compile o FFI com as features e defina as variáveis de ambiente:
  - Features: `guidelines-embed-openai` e/ou `guidelines-embed-ollama` (ex.: no script iOS já estão incluídas).
  - OpenAI: `PANTHER_OPENAI_API_KEY` (obrigatória), `PANTHER_OPENAI_BASE` (opcional), `PANTHER_OPENAI_EMBED_MODEL` (default `text-embedding-3-small`).
  - Ollama: `PANTHER_OLLAMA_BASE` (default `http://127.0.0.1:11434`), `PANTHER_OLLAMA_EMBED_MODEL` (default `nomic-embed-text`).

5) Posso salvar e recarregar um índice de diretrizes?
- Sim. O FFI expõe persistência simples via KeyValueStore (se “storage” estiver habilitado): `panther_guidelines_save_json(name, json)` e `panther_guidelines_load(name)`.

6) Como funcionam Storage e Logs?
- O Storage (in‑memory ou sled) permite salvar métricas ou blobs simples (como diretrizes). Logs em memória capturam eventos de chamada (e.g., “generate called”).
- FFI: `panther_storage_*` e `panther_logs_get|panther_logs_get_recent`.

7) Como estimar custo e tokens?
- FFI: `panther_token_count(text)` (tokenizador simples por espaço) e `panther_calculate_cost(tokens_in, tokens_out, provider_name, rules_json)`.
- O JSON de “rules” pode ser uma lista de objetos `{match|provider, usd_per_1k_in, usd_per_1k_out}`. O `match` aceita prefixos (ex.: `openai:gpt-4o`).

8) Métricas de conteúdo e viés
- Métricas como BLEU, ROUGE‑L, fluência, diversidade e plágio (baseline) via `panther_metrics_*`.
- Bias: `panther_bias_detect(samples_json)` retorna score de viés e estatísticas de grupos.

9) O que são as features (flags) do FFI e para que servem?
- `metrics-inmemory`/`metrics-prometheus`: integrações de métricas
- `storage-inmemory`/`storage-sled`: KeyValueStore
- `validation`, `validation-openai`, `validation-ollama`, `validation-anthropic`, `validation-async`: validação de LLMs
- `guidelines-embed-openai`, `guidelines-embed-ollama`: embeddings na Similarity
- `blockchain-eth`: ancoragem de proof on‑chain
- `agents*`: APIs de agentes (opcional)

10) iOS: como atualizar o XCFramework e resolver “Undefined symbol”? 
- Rode o script com as features corretas:
```
FEATURES="metrics-inmemory storage-inmemory validation validation-openai validation-ollama guidelines-embed-openai guidelines-embed-ollama" \
  ./scripts/ios_refresh_framework.sh
```
- Caminhos de saída:
  - `dist/<version>/ios/PantherSDK.xcframework`
  - `samples/swift/frameworkIOS/PantherSDKIOS.xcframework` (copiado para o sample)
- Verifique symbols:
```
nm -gU samples/swift/frameworkIOS/PantherSDKIOS.xcframework/ios-arm64-simulator/libpanther_ffi.a | rg panther_guidelines
```

11) iOS: “Cannot find type … in scope” ou problemas do SourceKit
- Confirme Target Membership dos arquivos no projeto Xcode (os novos arquivos devem pertencer ao alvo do app).
- Faça Clean Build Folder e reabra o projeto se necessário.

12) Swift sample: navegação e sessão
- O sample usa abas (Providers, Validate, Guidelines).
- “Providers” salva a sessão (via `AppSession` + `UserDefaults`). “Validate” e “Guidelines” consomem essa sessão.

13) React Native/Flutter/Kotlin: onde ficam as bridges?
- RN: `samples/react_native` (Panther.ts, módulos iOS/Android nativos)
- Flutter: `samples/flutter/lib/ffi.dart`
- Kotlin: `samples/kotlin/android/app/src/main/cpp/panther_jni.c` + `PantherBridge.kt`

14) Métricas Prometheus
- O backend Python expõe métricas via `prometheus_client` (opcional). Exportador Rust está no roadmap.

15) Segurança e chaves
- Use variáveis de ambiente para chaves. Não comite chaves em código/fonte.

