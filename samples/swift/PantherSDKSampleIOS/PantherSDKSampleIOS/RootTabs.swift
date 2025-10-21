import SwiftUI

struct RootTabs: View {
    @EnvironmentObject private var session: AppSession
    var body: some View {
        TabView {
            ProvidersScreen()
                .tabItem { Label("LLMs", systemImage: "gear") }
            ContentView()
                .tabItem { Label("Validate", systemImage: "checkmark.shield") }
            GuidelinesScreen()
                .tabItem { Label("Guidelines", systemImage: "list.bullet.rectangle") }
        }
        .onAppear { session.load() }
    }
}

struct ProvidersScreen: View {
    @EnvironmentObject private var session: AppSession
    @State private var showCostRules: Bool = false
    @FocusState private var advProvidersFocused: Bool
    @State private var showHelp: Bool = false
    @State private var showJsonAlert: Bool = false
    @State private var jsonAlertText: String = ""
    // JSON validation helpers for Advanced LLMs
    private var advTrimmed: String { session.advancedProvidersJSON.trimmingCharacters(in: .whitespacesAndNewlines) }
    private var advIsEmpty: Bool { advTrimmed.isEmpty }
    private var advIsValid: Bool {
        guard !advIsEmpty, let data = advTrimmed.data(using: .utf8) else { return false }
        return (try? JSONSerialization.jsonObject(with: data)) != nil
    }
    var body: some View {
        NavigationView {
            Form {
                Section(header: Text("LLM (Single)")) {
                    Picker("Type", selection: $session.provider) {
                        ForEach(AppSession.Provider.allCases, id: \.self) { Text(String(describing: $0).capitalized).tag($0) }
                    }
                    .pickerStyle(.segmented)
                }
                if session.provider == .openai {
                    Section(header: Text("OpenAI")) {
                        TextField("API Key", text: $session.openAIKey).textInputAutocapitalization(.never).disableAutocorrection(true)
                        TextField("Base URL", text: $session.openAIBase)
                        ModelPicker(title: "Model", presets: CostRules.openAIModels, selection: $session.openAIModel)
                    }
                } else if session.provider == .ollama {
                    Section(header: Text("Ollama")) {
                        TextField("Base URL", text: $session.ollamaBase)
                        ModelPicker(title: "Model", presets: CostRules.ollamaModels, selection: $session.ollamaModel)
                    }
                } else if session.provider == .anthropic {
                    Section(header: Text("Anthropic")) {
                        TextField("API Key", text: $session.anthropicKey).textInputAutocapitalization(.never).disableAutocorrection(true)
                        TextField("Base URL", text: $session.anthropicBase)
                        ModelPicker(title: "Model", presets: CostRules.anthropicModels, selection: $session.anthropicModel)
                    }
                } else {
                    Section { Text("Usando providers de ambiente (Default)").font(.caption).foregroundColor(.secondary) }
                }

                // Advanced: allow multi-provider JSON override used by Validate -> Multi/With Proof
                Section(header: Text("LLMs (JSON avançado)")) {
                    Toggle("Usar LLMs JSON (avançado)", isOn: $session.useAdvancedProvidersJSON)
                    if session.useAdvancedProvidersJSON {
                        TextEditor(text: $session.advancedProvidersJSON)
                            .focused($advProvidersFocused)
                            .font(.system(.footnote, design: .monospaced))
                            .frame(minHeight: 120)
                            .padding(6)
                            .background(Color(UIColor.secondarySystemBackground))
                            .cornerRadius(8)
                        if advIsEmpty {
                            Text("JSON vazio — Multi usará o LLM Single (fallback)")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        } else if !advIsValid {
                            Text("JSON inválido — não é JSON válido")
                                .font(.caption)
                                .foregroundColor(.red)
                        } else {
                            Text("JSON válido")
                                .font(.caption)
                                .foregroundColor(.green)
                        }
                        VStack(alignment: .leading, spacing: 8) {
                            Button {
                                // Exemplo pronto: OpenAI chatgpt-5 + gpt-4o
                                let base = session.openAIBase.isEmpty ? "https://api.openai.com" : session.openAIBase
                                let key = session.openAIKey
                                let arr: [[String: String]] = [
                                    ["type": "openai", "api_key": key, "base_url": base, "model": "chatgpt-5"],
                                    ["type": "openai", "api_key": key, "base_url": base, "model": "gpt-4o"]
                                ]
                                if let data = try? JSONSerialization.data(withJSONObject: arr, options: [.prettyPrinted]), let s = String(data: data, encoding: .utf8) {
                                    session.useAdvancedProvidersJSON = true
                                    session.advancedProvidersJSON = s
                                    session.save()
                                    jsonAlertText = "Exemplo preenchido. JSON avançado habilitado."
                                    showJsonAlert = true
                                }
                            } label: {
                                Label("Preencher Exemplo", systemImage: "wand.and.stars")
                                    .frame(maxWidth: .infinity)
                            }
                            .buttonStyle(.borderedProminent)

                            Button {
                                _ = session.save()
                                jsonAlertText = "JSON salvo"
                                showJsonAlert = true
                            } label: {
                                Label("Salvar JSON", systemImage: "square.and.arrow.down")
                            }
                            .buttonStyle(.bordered)
                        }
                        Text("O JSON acima será usado nos modos Multi/With Proof. Deixe desativado para usar a seleção simples desta tela.")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }
                Section(header: Text("Custos")) {
                    Button("Editar Tabela") { showCostRules = true }
                }
                Section { Button("Salvar sessão") { session.save() } }
            }
            .simultaneousGesture(TapGesture().onEnded { advProvidersFocused = false })
            .navigationTitle("LLMs")
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button { showHelp = true } label: { Image(systemName: "questionmark.circle") }
                }
                ToolbarItemGroup(placement: .keyboard) {
                    Spacer()
                    Button("Fechar") { advProvidersFocused = false }
                }
            }
            .alert(jsonAlertText, isPresented: $showJsonAlert) { Button("OK", role: .cancel) {} }
        }
        .sheet(isPresented: $showCostRules) {
            NavigationView {
                CostRulesEditor()
            }
        }
        .sheet(isPresented: $showHelp) {
            NavigationView { HelpView() }
        }
    }
}

private struct CostRulesEditor: View {
    @State private var costRulesJson: String = CostRules.defaultJSON
    @Environment(\.dismiss) private var dismiss
    var body: some View {
        VStack(alignment: .leading) {
            TextEditor(text: $costRulesJson)
                .font(.system(.footnote, design: .monospaced))
                .padding(8)
                .background(Color(UIColor.secondarySystemBackground))
                .cornerRadius(8)
            HStack {
                Button("Restaurar Padrão") { costRulesJson = CostRules.defaultJSON }
                Spacer()
                Button("Fechar") { dismiss() }.buttonStyle(.borderedProminent)
            }
        }
        .padding()
        .navigationTitle("Tabela de Custos")
    }
}

struct GuidelinesScreen: View {
    @State private var guidelinesURL: String = ""
    @State private var useCustomGuidelines: Bool = false
    @State private var customGuidelines: String = ""
    @State private var simRows: [SimilarityRow] = []
    @State private var simMethod: ContentView.SimMethod = .hybrid
    @State private var indexName: String = "default"
    @State private var prompt: String = "Explique recomendações seguras de medicamentos na gravidez."
    @State private var simNotice: String = ""

    @State private var showHelp: Bool = false
    var body: some View {
        NavigationView {
            ScrollView {
                VStack(alignment: .leading, spacing: 12) {
                    SectionHeader("Prompt para similaridade")
                    TextEditor(text: $prompt)
                        .frame(minHeight: 70)
                        .padding(8)
                        .background(Color(UIColor.secondarySystemBackground))
                        .cornerRadius(8)
                    SectionHeader("Diretrizes")
                    Toggle("Usar diretrizes customizadas (JSON)", isOn: $useCustomGuidelines)
                    if useCustomGuidelines {
                        TextEditor(text: $customGuidelines)
                            .frame(minHeight: 90)
                            .padding(8)
                            .background(Color(UIColor.secondarySystemBackground))
                            .cornerRadius(8)
                            .font(.caption)
                    } else {
                        Text("ANVISA (padrão embutido)").font(.caption).foregroundColor(.secondary)
                    }
                    HStack {
                        TextField("https://…/guidelines.json", text: $guidelinesURL).textFieldStyle(.roundedBorder)
                        Button("Carregar") { loadGuidelinesFromURL() }
                        Button("Fetch + scores") { fetchAndScore() }
                    }
                    HStack {
                        TextField("Nome do índice", text: $indexName).textFieldStyle(.roundedBorder)
                        Button("Salvar índice") { saveIndex() }
                        Button("Carregar índice") { loadIndex() }
                    }
                    Picker("Método", selection: $simMethod) {
                        ForEach(ContentView.SimMethod.allCases, id: \.self) { Text($0.rawValue).tag($0) }
                    }
                    .pickerStyle(.menu)
                    Text(methodDescription(simMethod))
                        .font(.caption)
                        .foregroundColor(.secondary)
                    if !simRows.isEmpty {
                        SectionHeader("Similarity Scores")
                        VStack(alignment: .leading, spacing: 8) {
                            ForEach(simRows) { r in
                                HStack {
                                    Text(r.topic).font(.subheadline)
                                    Spacer()
                                    let detail = "(" + (r.bow.map { String(format: "bow %.3f", $0) } ?? "") + (r.jaccard.map { String(format: ", jac %.3f", $0) } ?? "") + ")"
                                    Text(String(format: "%.3f %@", r.score, detail)).monospaced()
                                }
                                .padding(8)
                                .background(Color(UIColor.tertiarySystemBackground))
                                .cornerRadius(6)
                            }
                        }
                        if !simNotice.isEmpty { Text(simNotice).font(.caption).foregroundColor(.secondary) }
                    }
                }
                .padding()
            }
            .navigationTitle("Guidelines")
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button { showHelp = true } label: { Image(systemName: "questionmark.circle") }
                }
            }
        }
        .sheet(isPresented: $showHelp) { NavigationView { HelpView() } }
    }

    private func loadGuidelinesFromURL() {
        PantherBridge.loadGuidelinesFromURL(guidelinesURL) { s in
            guard let s else { return }
            DispatchQueue.main.async { self.customGuidelines = s; self.useCustomGuidelines = true }
        }
    }
    private func fetchAndScore() {
        PantherBridge.loadGuidelinesFromURL(guidelinesURL) { s in
            guard let s = s else { return }
            let n = PantherBridge.guidelinesIngest(json: s)
            guard n > 0 else { return }
            var method = simMethod.rawValue.lowercased()
            if simMethod == .embedOpenAI { method = "embed-openai" }
            if simMethod == .embedOllama { method = "embed-ollama" }
            if simMethod == .embedOpenAI || simMethod == .embedOllama {
                _ = PantherBridge.guidelinesBuildEmbeddings(method: method)
            }
            let out = PantherBridge.guidelinesScores(query: prompt, topK: 5, method: method)
            if let data = out.data(using: .utf8),
               let arr = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]] {
                let rows = arr.compactMap { d -> SimilarityRow? in
                    let t = d["topic"] as? String ?? ""
                    let sc = d["score"] as? Double ?? 0
                    let bow = d["bow"] as? Double
                    let jac = d["jaccard"] as? Double
                    return SimilarityRow(topic: t, score: sc, bow: bow, jaccard: jac)
                }
                DispatchQueue.main.async { self.simRows = rows }
            }
        }
    }

    private func methodDescription(_ m: ContentView.SimMethod) -> String {
        switch m {
        case .bow: return "BOW: Bag‑of‑Words com cosseno (local/rápido)."
        case .jaccard: return "Jaccard: similaridade de conjuntos de tokens (local/rápido)."
        case .hybrid: return "Hybrid: combinação BOW + Jaccard (equilíbrio de precisão/recall)."
        case .embedOpenAI: return "Embeddings (OpenAI): similaridade vetorial; requer PANTHER_OPENAI_API_KEY."
        case .embedOllama: return "Embeddings (Ollama): similaridade vetorial local; requer PANTHER_OLLAMA_BASE."
        }
    }

    private func saveIndex() {
        guard !indexName.trimmingCharacters(in: .whitespaces).isEmpty else { return }
        let json = useCustomGuidelines ? customGuidelines : (try? String(contentsOf: URL(string: guidelinesURL)!)) ?? ""
        guard !json.isEmpty else { return }
        _ = PantherBridge.guidelinesSave(name: indexName, json: json)
    }

    private func loadIndex() {
        guard !indexName.trimmingCharacters(in: .whitespaces).isEmpty else { return }
        let n = PantherBridge.guidelinesLoad(name: indexName)
        if n > 0 { fetchAndScore() }
    }
}

// Fallback HelpView (in case the separate file isn't part of target)
struct HelpView: View {
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 12) {
                SectionHeader("LLMs")
                Text("Configure o LLM 'Single' e use 'LLMs (JSON avançado)' para Multi/Proof.")
                    .font(.subheadline)
                SectionHeader("LLMs (JSON avançado)")
                Text("Exemplos de listas para Multi/Proof:")
                    .font(.subheadline)
                Text("OpenAI — chatgpt-5 + gpt-4o").font(.caption).foregroundColor(.secondary)
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
                Text("Mix — OpenAI + Ollama (local)").font(.caption).foregroundColor(.secondary)
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
                Text("OpenAI + Anthropic").font(.caption).foregroundColor(.secondary)
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
                SectionHeader("Validate")
                Text("Single: 1 LLM. Multi: todos do JSON. With Proof: Multi + hash de auditoria.")
                    .font(.subheadline)
                SectionHeader("Guidelines")
                Text("Sem JSON: ANVISA. Com JSON: array {topic, expected_terms[]}.")
                    .font(.subheadline)
                SectionHeader("Similarity")
                Text("BOW/Jaccard/Hybrid locais; Embeddings exigem credenciais/endpoint.")
                    .font(.subheadline)
                SectionHeader("Custos")
                Text("Tabela de preços por 1k tokens; usada para estimativa em Validate.")
                    .font(.subheadline)
            }
            .padding()
        }
        .navigationTitle("Ajuda")
    }
}
