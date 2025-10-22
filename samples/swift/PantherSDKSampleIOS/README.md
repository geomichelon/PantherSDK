<<<<<<< HEAD
PantherSDK Swift Sample — Quick Guide

Validate — Modes
- Single: runs only the provider/model selected in Providers.
- Multi: runs all providers listed in “LLMs (Advanced JSON)” in parallel (falls back to Single if the JSON is empty).
- With Proof: same as Multi and also returns `{results, proof}` with a deterministic combined hash for auditability (Anchor/Status).

Scores & Compliance
- Adherence score (per‑provider): 0–100% based on presence of expected_terms from the active guideline in the response. 100% = all terms present; 0% = none or error.
- Trust Index (run‑level): mean adherence of successful rows × safety‑coverage ratio over the combined text (consultation, risks/contraindications, lowest effective dose, prenatal vitamins/folic acid, vaccinations: flu shot/Tdap, labeling: FDA/PLLR). Result ∈ [0,1].
  - Multi‑LLM note: coverage is computed on the merged text of successful rows (can be high with more rows), while the mean adherence may decrease if several rows are mid/low — e.g., coverage ≈ 0.83 and Trust ≈ 0.30.
  - Single‑LLM rule: if the run returns a single result (one provider), Trust equals adherence (coverage factor not applied).

Guidelines
- Built‑in default: ANVISA example (see `crates/panther-validation/guidelines/anvisa.json`).
- Custom JSON: toggle “Use custom guidelines (JSON)” and paste an array of `{topic, expected_terms[]}`.
- Guidelines URL (http/https, also `s3://` and `gs://` mapped):
  - Load: downloads the JSON and fills the Custom Guidelines editor (enables the toggle so Validate uses it).
  - Fetch + scores: downloads the JSON and computes Similarity for the current prompt using the selected method. Shows Top‑K under “Similarity Scores” (doesn’t change the active guideline unless you enable the toggle or press Load).

Similarity (local vs external)
- Local (offline):
  - BOW — Bag‑of‑Words cosine using normalized term‑frequency vectors.
  - Jaccard — set overlap |A∩B|/|A∪B|.
  - Hybrid — 0.6×BOW + 0.4×Jaccard.
- External endpoints (optional): Embeddings (OpenAI/Ollama) require credentials/endpoint; use only if you need vector similarity.

Advanced JSON (Multi)
- Enable “LLMs (Advanced JSON)” and click “Fill Example (OpenAI — all presets)” to populate OpenAI models or paste your own array.
- Example (OpenAI + Ollama):
```
[
  {"type": "openai", "api_key": "sk-…", "base_url": "https://api.openai.com", "model": "gpt-4o-mini"},
  {"type": "ollama", "base_url": "http://127.0.0.1:11434", "model": "llama3"}
]
```

Proof (Anchoring via API)
- Run in “With Proof”, set API Base (e.g., `http://127.0.0.1:8000`), tap “Anchor” and “Status”.
- Server needs: `PANTHER_ETH_RPC`, `PANTHER_PROOF_CONTRACT`, `PANTHER_ETH_PRIVKEY` to anchor on chain (see repo README).

Monitoring — Prometheus Pushgateway
- In the Compliance section, enable “Push metrics to Prometheus (Pushgateway)”.
- Default endpoint: http://127.0.0.1:9091. If a root URL is provided, the app posts to /metrics/job/panther/instance/ios.
- Metrics pushed:
  - panther_trust_index (gauge), panther_coverage_ratio (gauge)
  - Per provider (rows with score > 0): panther_adherence_score{provider} (0..1), panther_latency_ms{provider}, panther_cost_usd{provider}
- Quick test:
  - docker run -p 9091:9091 prom/pushgateway
  - Enable the toggle, click “Generate Compliance” or “Push now”
  - Open http://127.0.0.1:9091/metrics to see the series

Notes
- Costs shown are estimates using the editable price table (per 1k tokens).
- Local methods protect privacy (no network calls). Embeddings require credentials.

Demo Video
- If you place a demo video at the repo path `docs/media/demo.mp4`, GitHub will render it in the root README. You can also view it from this README via the relative link below:

<video src="../../../docs/media/demo.mp4" width="720" controls muted playsinline>
  Your browser does not support the video tag. See the link below.
</video>

- Direct link: ../../../docs/media/demo.mp4
=======
PantherSDK Swift Sample — Anthropic Preset

Overview
- This sample demonstrates Panther validation (single/multi/with proof) with provider presets. In addition to OpenAI and Ollama, an Anthropic preset is available.

Anthropic Preset
- UI fields: Anthropic API Key, Base URL (default https://api.anthropic.com), and model (e.g., claude-3-5-sonnet-latest).
- Modes:
  - Single: under Provider = Anthropic, the sample builds a single-entry providers JSON and runs validation.
  - Multi/With Proof: Provider selection contributes an Anthropic entry in the providers array; proof hash is shown in "With Proof".

Providers JSON (example)
```
[
  {"type": "anthropic", "api_key": "sk-...", "base_url": "https://api.anthropic.com", "model": "claude-3-5-sonnet-latest"}
]
```

Notes
- The Core/FFI must be built with Anthropic features enabled to run Anthropic calls:
  - Sync: `-p panther-ffi --features "validation validation-anthropic"`
  - Async: `-p panther-ffi --features "validation validation-async validation-anthropic-async"`
- Costs shown are estimates based on the editable rules in the sample.
- For local testing, OpenAI and Ollama presets remain available.

>>>>>>> origin/main
