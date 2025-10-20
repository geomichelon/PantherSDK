import SwiftUI

struct RootTabs: View {
    @EnvironmentObject private var session: AppSession
    var body: some View {
        TabView {
            ProvidersScreen()
                .tabItem { Label("Providers", systemImage: "gear") }
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
    var body: some View {
        NavigationView {
            Form {
                Section(header: Text("Provider")) {
                    Picker("Type", selection: $session.provider) {
                        ForEach(AppSession.Provider.allCases, id: \.self) { Text(String(describing: $0).capitalized).tag($0) }
                    }
                    .pickerStyle(.segmented)
                }
                if session.provider == .openai {
                    Section(header: Text("OpenAI")) {
                        TextField("API Key", text: $session.openAIKey).textInputAutocapitalization(.never).disableAutocorrection(true)
                        TextField("Base URL", text: $session.openAIBase)
                        Picker("Model", selection: $session.openAIModel) {
                            ForEach(CostRules.openAIModels, id: \.self) { Text($0).tag($0) }
                        }
                    }
                } else if session.provider == .ollama {
                    Section(header: Text("Ollama")) {
                        TextField("Base URL", text: $session.ollamaBase)
                        Picker("Model", selection: $session.ollamaModel) {
                            ForEach(CostRules.ollamaModels, id: \.self) { Text($0).tag($0) }
                        }
                    }
                } else if session.provider == .anthropic {
                    Section(header: Text("Anthropic")) {
                        TextField("API Key", text: $session.anthropicKey).textInputAutocapitalization(.never).disableAutocorrection(true)
                        TextField("Base URL", text: $session.anthropicBase)
                        Picker("Model", selection: $session.anthropicModel) {
                            ForEach(CostRules.anthropicModels, id: \.self) { Text($0).tag($0) }
                        }
                    }
                } else {
                    Section { Text("Usando providers de ambiente (Default)").font(.caption).foregroundColor(.secondary) }
                }
                Section(header: Text("Custos")) {
                    Button("Editar Tabela") { showCostRules = true }
                }
                Section { Button("Salvar sessão") { session.save() } }
            }
            .navigationTitle("Providers")
        }
        .sheet(isPresented: $showCostRules) {
            NavigationView {
                CostRulesEditor()
            }
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
                    }.pickerStyle(.segmented)
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
        }
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
            if simMethod == .embedOpenAI || simMethod == .embedOllama { _ = PantherBridge.guidelinesBuildEmbeddings(method: method) }
            let out = PantherBridge.guidelinesScores(query: prompt, topK: 5, method: method)
            if let data = out.data(using: .utf8), let arr = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]] {
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

