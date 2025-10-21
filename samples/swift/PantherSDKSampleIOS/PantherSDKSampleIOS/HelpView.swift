import SwiftUI

struct HelpView: View {
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                SectionHeader("LLMs")
                Text("Configure aqui o provedor e o modelo a serem usados no modo Single. Para rodar múltiplos LLMs em paralelo, use 'LLMs (JSON avançado)'.")
                    .font(.subheadline)

                SectionHeader("LLMs (JSON avançado)")
                Text("Lista manual dos LLMs para os modos Multi/With Proof. Exemplos:")
                    .font(.subheadline)
                // OpenAI (chatgpt-5 + gpt-4o)
                Text("OpenAI — chatgpt-5 + gpt-4o")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Text("""
[
  { "type": "openai", "api_key": "sk-…", "base_url": "https://api.openai.com", "model": "chatgpt-5" },
  { "type": "openai", "api_key": "sk-…", "base_url": "https://api.openai.com", "model": "gpt-4o" }
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

                SectionHeader("Validate — modos")
                Text("Single: executa apenas o LLM selecionado.\nMulti: executa em paralelo todos os LLMs do JSON avançado; se vazio, usa o Single.\nWith Proof: igual ao Multi, mas retorna também um 'proof' (hash combinado) para auditoria.")
                    .font(.subheadline)

                SectionHeader("Diretrizes (Guidelines)")
                Text("Sem JSON: usa ANVISA embutido.\nCom JSON: cola seu conjunto (array de objetos {topic, expected_terms[]}). O score de aderência é calculado por termos esperados presentes/ausentes.")
                    .font(.subheadline)

                SectionHeader("Similarity (aba Guidelines)")
                Text("BOW/Jaccard/Hybrid: métodos locais e rápidos.\nEmbeddings (OpenAI/Ollama): similaridade vetorial; requer credenciais/endpoint.")
                    .font(.subheadline)

                SectionHeader("Custos (estimativa)")
                Text("A tabela de custos define preço por 1k tokens para cada provedor/modelo (campo 'match'). A tela Validate exibe custo estimado por resposta.")
                    .font(.subheadline)

                SectionHeader("Proof (auditoria)")
                Text("Com 'With Proof', o SDK calcula um hash combinado das entradas (prompt, LLMs, diretrizes) e saídas (resultados). É possível ancorar esse hash via API opcional.")
                    .font(.subheadline)
            }
            .padding()
        }
        .navigationTitle("Ajuda")
    }
}
