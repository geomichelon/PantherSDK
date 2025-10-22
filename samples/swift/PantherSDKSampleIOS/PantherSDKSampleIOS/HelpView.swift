import SwiftUI

struct HelpView: View {
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                Text("LLMs").font(.headline)
                Text("Configure the provider and model used in Single mode. To run multiple LLMs in parallel, use 'LLMs (Advanced JSON)'.")
                    .font(.subheadline)

                Text("LLMs (Advanced JSON)").font(.headline)
                Text("Manual list of LLMs for Multi/With Proof. Examples:")
                    .font(.subheadline)
                // OpenAI (chatgpt-5 + gpt-4o)
                Text("OpenAI — gpt-4o + gpt-4.1-mini")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Text("""
[
  { "type": "openai", "api_key": "sk-…", "base_url": "https://api.openai.com", "model": "gpt-4o" },
  { "type": "openai", "api_key": "sk-…", "base_url": "https://api.openai.com", "model": "gpt-4.1-mini" }
]
""")
                .font(.system(.footnote, design: .monospaced))
                .padding(8)
                .background(Color.secondary.opacity(0.12))
                .cornerRadius(8)

                // Mix OpenAI + Ollama
                Text("Mix — OpenAI + Ollama (local)")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Text("""
[
  { "type": "openai", "api_key": "sk-…", "base_url": "https://api.openai.com", "model": "gpt-4o-mini" },
  { "type": "ollama",  "base_url": "http://127.0.0.1:11434",          "model": "llama3" }
]
""")
                .font(.system(.footnote, design: .monospaced))
                .padding(8)
                .background(Color.secondary.opacity(0.12))
                .cornerRadius(8)

                // OpenAI + Anthropic
                Text("OpenAI + Anthropic")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Text("""
[
  { "type": "openai",    "api_key": "sk-…",     "base_url": "https://api.openai.com",   "model": "gpt-4.1-mini" },
  { "type": "anthropic", "api_key": "sk-ant-…", "base_url": "https://api.anthropic.com", "model": "claude-3-5-sonnet-latest" }
]
""")
                .font(.system(.footnote, design: .monospaced))
                .padding(8)
                .background(Color.secondary.opacity(0.12))
                .cornerRadius(8)

                Text("Validate — Modes").font(.headline)
                Text("Single — Runs only the provider/model selected in the Providers tab. Good for quick checks and lowest cost/latency.")
                    .font(.subheadline)
                Text("Multi — Runs all providers listed in ‘LLMs (Advanced JSON)’ in parallel (if empty, falls back to Single). Useful for side‑by‑side comparison and ranking by adherence.")
                    .font(.subheadline)
                Text("With Proof — Same as Multi, but also returns {results, proof}. The proof contains a deterministic combined hash of inputs and outputs, enabling the Anchor/Status buttons for auditability.")
                    .font(.subheadline)

                Text("Scores & Compliance").font(.headline)
                Text("Adherence score — Per‑provider percentage (0–100) based on presence of expected_terms in the response. 100% = all terms present; 0% = none (or upstream error).")
                    .font(.subheadline)
                Text("Trust Index — Average adherence (only rows with score > 0) multiplied by a safety‑coverage ratio computed over the combined text (consultation, risks/contraindications, lowest effective dose, prenatal vitamins/folic acid, vaccinations: flu shot/Tdap, labeling: FDA/PLLR). Result ∈ [0,1].")
                    .font(.subheadline)
                Text("Single‑LLM rule — If the run returns a single result (one provider), Trust equals adherence (coverage factor not applied).")
                    .font(.subheadline)
                Text("Multi‑LLM formula — Trust = mean(adherence/100 of valid rows) × coverage_ratio.")
                    .font(.subheadline)
                Text("Examples — (a) 33/33/33/33 with coverage 1.0 ⇒ 33%. (b) 33/33/33/33 with coverage 0.83 ⇒ ≈27.4%. (c) 56/33/33/33 with coverage 1.0 ⇒ 38.75%. (d) same with 0.83 ⇒ ≈32.2%.")
                    .font(.subheadline)
                Text("Multi‑LLM note — Coverage uses the merged text of successful rows (often high with more rows), while the average adherence can go down if several rows are mid/low. This explains cases like coverage ≈ 0.83 but Trust ≈ 0.30.")
                    .font(.subheadline)

                Text("Prompt Strategy").font(.headline)
                Text("Baseline — Sends the prompt as‑is. Lowest cost/latency; response ‘livre’. Adherence/coverage dependem do seu prompt.")
                    .font(.subheadline)
                Text("Structured — Adds best‑practice instructions and requests sections (General Recommendations, Risks, Dosing, Avoid, Vaccinations) + short summary. Tende a aumentar adherence e safety coverage, com custo moderado.")
                    .font(.subheadline)
                Text("Full (JSON) — Deterministic JSON with keys {summary, key_points[], risks[], recommendations[], contraindications[], checklist[]}. Máxima previsibilidade e fácil parsing; custo mais alto.")
                    .font(.subheadline)
                Text("Run All (3) — Runs Baseline, Structured and Full (JSON) and labels each card with its strategy for side‑by‑side comparison.")
                    .font(.subheadline)

                Text("Guidelines").font(.headline)
                Text("Without JSON: uses built-in ANVISA.\nWith JSON: paste your own set (array of objects {topic, expected_terms[]}). Adherence is computed by missing/present expected terms.")
                    .font(.subheadline)
                Text("Guidelines URL — Actions").font(.headline)
                Text("Load — Downloads the JSON from the URL (http/https; s3:// and gs:// are mapped to public endpoints) and fills the Custom Guidelines editor, enabling the toggle so Validate uses that JSON.")
                    .font(.subheadline)
                Text("Fetch + scores — Downloads the same JSON and computes Similarity for the current prompt using the selected method (BOW/Jaccard/Hybrid/Embeddings). Shows Top‑K under ‘Similarity Scores’. It does not change the active guideline unless you enable the custom toggle or press Load.")
                    .font(.subheadline)

                Text("Similarity (Guidelines tab)").font(.headline)
                Text("BOW/Jaccard/Hybrid: local & fast.\nEmbeddings (OpenAI/Ollama): vector similarity; requires credentials/endpoint.")
                    .font(.subheadline)

                Text("Similarity methods").font(.headline)
                Text("BOW — Bag-of-Words cosine using term-frequency vectors (normalized). Runs locally (offline).")
                    .font(.subheadline)
                Text("Jaccard — Set overlap of tokens: |A∩B|/|A∪B|. Runs locally (offline).")
                    .font(.subheadline)
                Text("Hybrid — 0.6×BOW + 0.4×Jaccard for balanced matching. Runs locally (offline).")
                    .font(.subheadline)

                Text("Costs (estimate)").font(.headline)
                Text("The cost table defines price per 1k tokens per provider/model (match field). Validate shows estimated cost per response.")
                    .font(.subheadline)

                Text("Proof (auditing)").font(.headline)
                Text("With 'With Proof', the SDK computes a combined hash of inputs (prompt, LLMs, guidelines) and outputs (results). You can optionally anchor this via the API.")
                    .font(.subheadline)

                Text("Monitoring (Prometheus)").font(.headline)
                Text("Toggle ‘Push metrics to Prometheus (Pushgateway)’ in the Compliance section to export run‑level and per‑provider metrics.")
                    .font(.subheadline)
                Text("Endpoint — defaults to http://127.0.0.1:9091. If a root URL is provided, metrics are posted to /metrics/job/panther/instance/ios.")
                    .font(.subheadline)
                Text("Metrics sent — panther_trust_index, panther_coverage_ratio; and per provider: panther_adherence_score{provider}, panther_latency_ms{provider}, panther_cost_usd{provider}.")
                    .font(.subheadline)
                Text("Quick test — docker run -p 9091:9091 prom/pushgateway; enable the toggle; ‘Generate Compliance’ or ‘Push now’; then open /metrics on the Pushgateway.")
                    .font(.subheadline)
            }
            .padding()
        }
        .navigationTitle("Help")
    }
}
