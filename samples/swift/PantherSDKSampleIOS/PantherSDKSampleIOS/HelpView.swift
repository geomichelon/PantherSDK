import SwiftUI

struct HelpView: View {
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                SectionHeader("LLMs")
                Text("Configure the provider and model used in Single mode. To run multiple LLMs in parallel, use 'LLMs (Advanced JSON)'.")
                    .font(.subheadline)

                SectionHeader("LLMs (Advanced JSON)")
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
                .background(Color(UIColor.secondarySystemBackground))
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
                .background(Color(UIColor.secondarySystemBackground))
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
                .background(Color(UIColor.secondarySystemBackground))
                .cornerRadius(8)

                SectionHeader("Validate — Modes")
                Text("Single — Runs only the provider/model selected in the Providers tab. Good for quick checks and lowest cost/latency.")
                    .font(.subheadline)
                Text("Multi — Runs all providers listed in ‘LLMs (Advanced JSON)’ in parallel (if empty, falls back to Single). Useful for side‑by‑side comparison and ranking by adherence.")
                    .font(.subheadline)
                Text("With Proof — Same as Multi, but also returns {results, proof}. The proof contains a deterministic combined hash of inputs and outputs, enabling the Anchor/Status buttons for auditability.")
                    .font(.subheadline)

                SectionHeader("Prompt Strategy")
                Text("Baseline — Sends the prompt as‑is.")
                    .font(.subheadline)
                Text("Structured — Adds best‑practice instructions and requests sections (General Recommendations, Risks, Dosing, Avoid, Vaccinations) plus a short summary to improve coverage and organization.")
                    .font(.subheadline)
                Text("Full (JSON) — Also called ‘Comprehensive’. Roleplays a healthcare compliance assistant and requires a deterministic JSON output with keys {summary, key_points[], risks[], recommendations[], contraindications[], checklist[]}. Useful for parsing and deterministic reviews.")
                    .font(.subheadline)
                Text("Run All (3) — Executes Baseline, Structured, and Full (JSON) in sequence and shows results labeled by strategy for side‑by‑side comparison.")
                    .font(.subheadline)

                SectionHeader("Guidelines")
                Text("Without JSON: uses built-in ANVISA.\nWith JSON: paste your own set (array of objects {topic, expected_terms[]}). Adherence is computed by missing/present expected terms.")
                    .font(.subheadline)
                SectionHeader("Guidelines URL — Actions")
                Text("Load — Downloads the JSON from the URL (http/https; s3:// and gs:// are mapped to public endpoints) and fills the Custom Guidelines editor, enabling the toggle so Validate uses that JSON.")
                    .font(.subheadline)
                Text("Fetch + scores — Downloads the same JSON and computes Similarity for the current prompt using the selected method (BOW/Jaccard/Hybrid/Embeddings). Shows Top‑K under ‘Similarity Scores’. It does not change the active guideline unless you enable the custom toggle or press Load.")
                    .font(.subheadline)

                SectionHeader("Similarity (Guidelines tab)")
                Text("BOW/Jaccard/Hybrid: local & fast.\nEmbeddings (OpenAI/Ollama): vector similarity; requires credentials/endpoint.")
                    .font(.subheadline)

                SectionHeader("Similarity methods")
                Text("BOW — Bag-of-Words cosine using term-frequency vectors (normalized). Runs locally (offline).")
                    .font(.subheadline)
                Text("Jaccard — Set overlap of tokens: |A∩B|/|A∪B|. Runs locally (offline).")
                    .font(.subheadline)
                Text("Hybrid — 0.6×BOW + 0.4×Jaccard for balanced matching. Runs locally (offline).")
                    .font(.subheadline)

                SectionHeader("Costs (estimate)")
                Text("The cost table defines price per 1k tokens per provider/model (match field). Validate shows estimated cost per response.")
                    .font(.subheadline)

                SectionHeader("Proof (auditing)")
                Text("With 'With Proof', the SDK computes a combined hash of inputs (prompt, LLMs, guidelines) and outputs (results). You can optionally anchor this via the API.")
                    .font(.subheadline)
            }
            .padding()
        }
        .navigationTitle("Help")
    }
}
