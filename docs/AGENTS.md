ProofSeal Agents (Stage 6)

Overview
- Orquestrador assíncrono para o fluxo: Validar → Selar Prova → Ancorar → Registrar/Metricar.
- Implementado em Rust (crate `panther-agents`) e exposto via FFI (feature `agents`) e via API Python (`/agent/*`).

Build/Features
- FFI com agentes (providers síncronos):
  - `cargo build -p panther-ffi --features "agents agents-openai agents-ollama"`
- FFI com agentes (providers assíncronos):
  - `cargo build -p panther-ffi --features "agents agents-openai-async agents-ollama-async"`
- Com blockchain (ancoragem on-chain):
  - `cargo build -p panther-ffi --features "agents agents-openai agents-ollama blockchain-eth"`

Plan/DSL
- JSON simples com `type` e campos opcionais:
```json
{
  "type": "ValidateSealAnchor",
  "guidelines_json": null,
  "anchor": {
    "rpc_url": "https://...",
    "contract_addr": "0x...",
    "priv_key": "<hex>"
  },
  "timeouts_ms": { "validate_ms": 10000, "anchor_ms": 15000, "status_ms": 5000 },
  "retries": { "validate": 0, "anchor": 2, "status": 2 }
}
```

Input JSON
```json
{
  "prompt": "Explain insulin function",
  "providers": [
    { "type": "openai", "api_key": "...", "base_url": "https://api.openai.com", "model": "gpt-4o-mini" },
    { "type": "ollama", "base_url": "http://127.0.0.1:11434", "model": "llama3" }
  ],
  "salt": "abc"
}
```

FFI
- Símbolo: `panther_agent_run(plan_json, input_json) -> c_char*` (JSON do `AgentRunResult`).
- Habilite a feature `agents` (e features de providers/blockchain conforme necessário).

API (Python)
- Endpoints:
  - `POST /agent/run` body: `{ plan: {...} | null, input: {...}, async_run?: bool }`
  - `GET /agent/status?run_id=...`
  - `GET /agent/events?run_id=...`
  - `GET /agent/events/stream?run_id=...` (SSE incremental; emite eventos conforme ocorrem)
  - `POST /agent/start` → `{ run_id }`
  - `GET /agent/poll?run_id=...&cursor=0` → `{ events, done, cursor }`

Samples
- React Native sample inclui botão “Run Agent” e helpers HTTP em `samples/react_native/Panther.ts`:
  - `runAgent(plan, input, apiBase?, apiKey?, asyncRun=true)`
  - `getAgentStatus(runId, apiBase?, apiKey?)`
  - `getAgentEvents(runId, apiBase?, apiKey?)`
  - Exemplo de UI: `samples/react_native/AppSample.tsx`

Exemplo (curl)
```sh
curl -sX POST http://127.0.0.1:8000/agent/run \
  -H 'Content-Type: application/json' \
  -d '{
    "plan": { "type": "ValidateSealAnchor" },
    "input": {
      "prompt": "Explain insulin function",
      "providers": [ { "type": "openai", "api_key": "'$OPENAI_API_KEY'", "model": "gpt-4o-mini", "base_url": "https://api.openai.com" } ]
    },
    "async_run": true
  }'

curl -s "http://127.0.0.1:8000/agent/status?run_id=<id>"
curl -s "http://127.0.0.1:8000/agent/events?run_id=<id>" | jq
curl -Ns "http://127.0.0.1:8000/agent/events/stream?run_id=<id>"

# Incremental (start/poll)
curl -sX POST http://127.0.0.1:8000/agent/start \
  -H 'Content-Type: application/json' \
  -d '{ "plan": {"type":"ValidateSealAnchor"}, "input": {"prompt":"...","providers":[...]}}'
# => {"run_id":"r..."}
curl -s "http://127.0.0.1:8000/agent/poll?run_id=<id>&cursor=0" | jq
```

Persistência
- O backend persiste `proof_history` (Stage 3) e registra runs de agentes quando SQLite está habilitado (`PANTHER_SQLITE_PATH`).

Notas
- Timeouts e retries por etapa já aplicados no runner (validate/anchor/status), com backoff exponencial e jitter.
- SSE incremental: `/agent/events/stream` emite eventos em tempo real (sem esperar o fim). Headers customizados não são suportados pelo EventSource; se precisar de `X-API-Key`, use o fallback `/agent/poll`.
- Integração direta dos samples ainda não está ativa; use o backend HTTP por enquanto.

RN/Web (SSE)
- Web: `new EventSource('/agent/events/stream?run_id=...')`
- React Native: use uma lib (ex.: `react-native-sse`/`react-native-event-source`) ou o fallback por polling (`/agent/poll`).
