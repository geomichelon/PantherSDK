Changelog

Unreleased (0.1.3) — Stage 6 (Agents), Batch Eval, and New Metrics
- ProofSeal Agents (Stage 6):
  - New crate `panther-agents` (runner + events + retries/timeouts).
  - FFI: `panther_agent_run(plan_json, input_json)` behind feature `agents`.
  - Python API: `POST /agent/run`, `GET /agent/status`, `GET /agent/events`, `GET /agent/events/stream` (SSE).
  - Samples (React Native): helpers `runAgent/getAgentStatus/getAgentEvents` and a "Run Agent" button in `AppSample.tsx`.
- AI Evaluation CLI (Batch):
  - `panther-ai-eval` now supports batch via `--input {jsonl|csv}`, `--max-concurrency`, `--with-proof`.
  - Outputs: `results.jsonl` (one item per prompt) and `summary.csv` (per provider p50/p95/errors).
  - New flag `--metrics` to compute `rouge` and `factcheck` per item.
- New Metrics:
  - ROUGE‑L (F1) and Fact‑coverage (basic) in `panthersdk::domain::metrics`.
  - FFI exports: `panther_metrics_rouge_l`, `panther_metrics_fact_coverage`.
  - Python API `/metrics/evaluate` supports `metric=rouge|factcheck`.
- Core fix:
  - Avoid borrowing `&self` into `'static` in `Engine::generate_async` fallback (uses cloned parts in `spawn_blocking`).
- Docs:
  - Added `docs/AGENTS.md` and `docs/STATUS.md`; updated README (batch quickstart) and ROADMAP.

0.1.2 — Stage 2 (On‑chain anchoring) and UI buttons
- Solidity contract `ProofRegistry` added at `docs/contracts/ProofRegistry.sol`.
- Rust feature `blockchain-eth` with ethers‑rs client:
  - FFI: `panther_proof_anchor_eth`, `panther_proof_check_eth`.
- Python API endpoints:
  - `POST /proof/anchor`, `GET /proof/status` (server‑side, env‑driven).
- Samples:
  - Swift/Kotlin/Flutter: botão “Anchor Proof (API)” que chama o backend e mostra `tx_hash`.
  - React Native: helper `anchorProof(hash, apiBase?, apiKey?)` em `Panther.ts`.

0.1.1 — Stage 1 (Offline Proofs) and Packaging
- Added proof module (Stage 1) with SHA3-512 hashing and canonical JSON:
  - `compute_proof`, `verify_proof_local` in `crates/panther-validation`.
- New FFI exports (feature `validation`):
  - `panther_validation_run_multi_with_proof` → returns `{results, proof}`.
  - `panther_proof_compute` → computes proof from inputs/results.
- Python API extensions:
  - `POST /proof/compute` endpoint (uses FFI, Python fallback).
  - Improved dynamic loading of FFI across platforms.
- Samples updated to show proof hash (combined_hash):
  - Swift/Kotlin/Flutter/React Native now support “validate with proof” and display the hash.
- Packaging scripts:
  - New scripts in `scripts/release/` (iOS xcframework, Android AAR, Python wheels, WASM, headers, manifest/checksums).
- `panther_version_string` is dynamic (from crate version) and samples expose version helpers.

0.1.0 — Initial SDK
- Core Engine, FFI, Validation (OpenAI/Ollama), Python API, Samples, Storage (in-memory/sled), Metrics/Logs.
