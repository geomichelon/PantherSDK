import SwiftUI

struct ContentView: View {
    enum Mode: String, CaseIterable { case single = "Single", multi = "Multi", proof = "With Proof" }
    enum Provider: String, CaseIterable { case openai = "OpenAI", ollama = "Ollama", anthropic = "Anthropic", `default` = "Default" }
    @EnvironmentObject private var session: AppSession

    @State private var prompt: String = "Explique recomendações seguras de medicamentos na gravidez."
    @State private var mode: Mode = .single
    @State private var provider: Provider = .openai

    @State private var apiKey: String = ""
    @State private var openAIBase: String = "https://api.openai.com"
    @State private var openAIModel: String = "gpt-4o-mini"

    @State private var ollamaBase: String = "http://127.0.0.1:11434"
    @State private var ollamaModel: String = "llama3"

    @State private var anthropicBase: String = "https://api.anthropic.com"
    @State private var anthropicModel: String = "claude-3-5-sonnet-latest"
    @State private var anthropicKey: String = ""

    @State private var useCustomGuidelines: Bool = false
    @State private var customGuidelines: String = ""
    @FocusState private var guidelinesFocused: Bool
    private var isGuidelinesJSONValid: Bool {
        let s = customGuidelines.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !s.isEmpty, let data = s.data(using: .utf8) else { return false }
        return (try? JSONSerialization.jsonObject(with: data)) != nil
    }

    @State private var showCostRules: Bool = false
    @State private var costRulesJson: String = CostRules.defaultJSON

    @State private var isRunning: Bool = false
    @State private var results: [ValidationRow] = []
    @State private var proofJSON: String? = nil

    // Compliance & Proof extras
    @State private var complianceReport: String = ""
    @State private var trustIndex: Double = 0
    @State private var guidelinesURL: String = ""
    @State private var proofApiBase: String = "http://127.0.0.1:8000"
    @State private var lastAnchorResponse: String = ""
    @State private var simRows: [SimilarityRow] = []
    enum SimMethod: String, CaseIterable { case bow = "BOW", jaccard = "Jaccard", hybrid = "Hybrid", embedOpenAI = "Embed(OpenAI)", embedOllama = "Embed(Ollama)" }
    @State private var simMethod: SimMethod = .bow
    @State private var indexName: String = "default"
    @State private var simNotice: String = ""
    @State private var showHelp: Bool = false

    var body: some View {
        NavigationView {
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    SectionHeader("Prompt")
                    TextEditor(text: $prompt)
                        .frame(minHeight: 90)
                        .padding(8)
                        .background(Color(.secondarySystemBackground))
                        .cornerRadius(8)

                    SectionHeader("Execução")
                    Picker("Mode", selection: $mode) {
                        ForEach(Mode.allCases, id: \.self) { Text($0.rawValue).tag($0) }
                    }
                    .pickerStyle(.segmented)
                    // Provider configuration moved to Providers tab (session-backed)

                    SectionHeader("Diretrizes")
                    Toggle("Usar diretrizes customizadas (JSON)", isOn: $useCustomGuidelines)
                    if useCustomGuidelines {
                        TextEditor(text: $customGuidelines)
                            .focused($guidelinesFocused)
                            .frame(minHeight: 90)
                            .padding(8)
                            .background(Color(.secondarySystemBackground))
                            .cornerRadius(8)
                            .font(.caption)
                        let trimmed = customGuidelines.trimmingCharacters(in: .whitespacesAndNewlines)
                        if trimmed.isEmpty {
                            Text("JSON vazio — usando ANVISA (fallback)")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        } else if !isGuidelinesJSONValid {
                            Text("JSON inválido — não é JSON válido")
                                .font(.caption)
                                .foregroundColor(.red)
                        } else {
                            Text("JSON válido")
                                .font(.caption)
                                .foregroundColor(.green)
                        }
                    } else {
                        Text("ANVISA (padrão embutido)")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }

                    HStack {
                        SectionHeader("Custos (estimativa)")
                        Spacer()
                        Button("Editar Tabela") { showCostRules = true }
                            .buttonStyle(.bordered)
                    }

                    Button(action: runValidation) {
                        HStack { isRunning ? AnyView(AnyView(ProgressView())) : AnyView(Image(systemName: "checkmark.shield")); Text("Validate") }
                            .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(isRunning || prompt.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)

                    if !results.isEmpty {
                        SectionHeader("Resultados")
                        SummaryView(rows: results)
                        ForEach(results) { row in ResultCard(row: row) }
                    }

                    if let proof = proofJSON, mode == .proof {
                        SectionHeader("Prova")
                        Text(proof)
                            .font(.system(.footnote, design: .monospaced))
                            .padding(8)
                            .background(Color(.secondarySystemBackground))
                            .cornerRadius(8)

                        // Anchor proof via API
                        VStack(alignment: .leading, spacing: 8) {
                            Text("Anchor proof (Python API)").font(.subheadline).fontWeight(.semibold)
                            TextField("API Base (ex.: http://127.0.0.1:8000)", text: $proofApiBase).textFieldStyle(.roundedBorder)
                            HStack {
                                Button("Anchor") { anchorProof() }.buttonStyle(.borderedProminent)
                                Button("Status") { checkProofStatus() }
                            }
                            if !lastAnchorResponse.isEmpty {
                                Text(lastAnchorResponse)
                                    .font(.system(.footnote, design: .monospaced))
                                    .padding(8)
                                    .background(Color(.tertiarySystemBackground))
                                    .cornerRadius(6)
                            }
                        }
                        .padding(.top, 4)
                    }

                    // Compliance: bias/trust index
                    if !results.isEmpty {
                        SectionHeader("Compliance Report")
                        HStack { Button("Gerar Compliance") { generateCompliance() }; Spacer() }
                        if !complianceReport.isEmpty {
                            Text("Trust Index: \(String(format: "%.1f", trustIndex*100))%")
                                .font(.subheadline)
                                .foregroundColor(trustIndex >= 0.8 ? .green : (trustIndex >= 0.6 ? .orange : .red))
                            Text(complianceReport)
                                .font(.system(.footnote, design: .monospaced))
                                .padding(8)
                                .background(Color(.secondarySystemBackground))
                                .cornerRadius(8)
                        }
                    }

                    // Load Guidelines from URL helper
                    SectionHeader("Carregar Diretrizes por URL")
                    HStack {
                        TextField("https://…/guidelines.json", text: $guidelinesURL)
                            .textFieldStyle(.roundedBorder)
                        Button("Carregar") { loadGuidelinesFromURL() }
                        Button("Fetch + scores") { fetchAndScore() }
                    }

                    HStack {
                        TextField("Nome do índice", text: $indexName).textFieldStyle(.roundedBorder)
                        Button("Salvar índice") { saveIndex() }
                        Button("Carregar índice") { loadIndex() }
                    }

                    Picker("Método", selection: $simMethod) {
                        ForEach(SimMethod.allCases, id: \.self) { Text($0.rawValue).tag($0) }
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
                                    .background(Color(.tertiarySystemBackground))
                                    .cornerRadius(6)
                            }
                        }
                        if !simNotice.isEmpty {
                            Text(simNotice).font(.caption).foregroundColor(.secondary)
                        }
                    }
                }
                .padding()
            }
            .simultaneousGesture(TapGesture().onEnded { guidelinesFocused = false })
            .navigationTitle("Validate")
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button { showHelp = true } label: { Image(systemName: "questionmark.circle") }
                }
                ToolbarItemGroup(placement: .keyboard) {
                    Spacer()
                    Button("Fechar") { guidelinesFocused = false }
                }
            }
        }
        .sheet(isPresented: $showHelp) { NavigationView { HelpView() } }
        .onAppear {
            // Sync local provider fields from session to keep consistency across tabs
            switch session.provider {
            case .openai:
                provider = .openai
                apiKey = session.openAIKey
                openAIBase = session.openAIBase
                openAIModel = session.openAIModel
            case .ollama:
                provider = .ollama
                ollamaBase = session.ollamaBase
                ollamaModel = session.ollamaModel
            case .anthropic:
                provider = .anthropic
                anthropicBase = session.anthropicBase
                anthropicModel = session.anthropicModel
                anthropicKey = session.anthropicKey
            case .default:
                provider = .default
            }
            // Default to OpenAI embed if key present
            let hasKey = ProcessInfo.processInfo.environment["PANTHER_OPENAI_API_KEY"].map { !$0.isEmpty } ?? false
            if hasKey { simMethod = .embedOpenAI }
        }
        .sheet(isPresented: $showCostRules) {
            NavigationView { CostRulesInlineEditor(costRulesJson: $costRulesJson) }
        }
    }

    private func runValidation() {
        isRunning = true
        results = []
        proofJSON = nil
        DispatchQueue.global(qos: .userInitiated).async {
            let output: String
            switch mode {
            case .single:
                switch session.provider {
                case .openai:
                    output = callOpenAI(prompt: prompt, apiKey: session.openAIKey, model: session.openAIModel, base: session.openAIBase)
                case .ollama:
                    output = callOllama(prompt: prompt, base: session.ollamaBase, model: session.ollamaModel)
                case .anthropic:
                    let arr: [[String: String]] = [["type": "anthropic", "api_key": session.anthropicKey, "model": session.anthropicModel, "base_url": session.anthropicBase]]
                    let data = try? JSONSerialization.data(withJSONObject: arr)
                    let singleJSON = String(data: data ?? Data("[]".utf8), encoding: .utf8) ?? "[]"
                    output = callMulti(prompt: prompt, providersJSON: singleJSON, withProof: false)
                case .default:
                    output = callDefault(prompt: prompt)
                }
            case .multi:
                output = callMulti(prompt: prompt, providersJSON: session.providersJSON(), withProof: false)
            case .proof:
                output = callMulti(prompt: prompt, providersJSON: session.providersJSON(), withProof: true)
            }
            DispatchQueue.main.async {
                isRunning = false
                parseAndDisplay(output)
            }
        }
    }

    private func buildProvidersJSON() -> String {
        var arr: [[String: String]] = []
        switch provider {
        case .openai:
            arr.append(["type": "openai", "api_key": apiKey, "model": openAIModel, "base_url": openAIBase])
        case .ollama:
            arr.append(["type": "ollama", "model": ollamaModel, "base_url": ollamaBase])
        case .anthropic:
            arr.append(["type": "anthropic", "api_key": anthropicKey, "model": anthropicModel, "base_url": anthropicBase])
        case .default:
            break
        }
        let data = try? JSONSerialization.data(withJSONObject: arr)
        return String(data: data ?? Data("[]".utf8), encoding: .utf8) ?? "[]"
    }

    private func parseAndDisplay(_ s: String) {
        if mode == .proof, let data = s.data(using: .utf8),
           let obj = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any],
           let arr = obj["results"], let arrData = try? JSONSerialization.data(withJSONObject: arr),
           let json = try? JSONSerialization.jsonObject(with: arrData) as? [[String: Any]] {
            self.proofJSON = pretty(obj)
            self.results = rows(from: json)
            return
        }
        if let data = s.data(using: .utf8), let json = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]] {
            self.results = rows(from: json)
        }
    }

    private func rows(from arr: [[String: Any]]) -> [ValidationRow] {
        let tokensIn = PantherBridge.tokenCount(prompt)
        let rules = costRulesJson
        return arr.compactMap { r -> ValidationRow? in
            let name = r["provider_name"] as? String ?? "?"
            let score = r["adherence_score"] as? Double ?? 0
            let lat = r["latency_ms"] as? Int ?? 0
            let text = r["raw_text"] as? String ?? ""
            let tokensOut = PantherBridge.tokenCount(text)
            let cost = PantherBridge.calculateCost(tokensIn: tokensIn, tokensOut: tokensOut, provider: name, rules: rules)
            return ValidationRow(provider: name, score: score, latencyMs: lat, tokensIn: tokensIn, tokensOut: tokensOut, costUSD: cost, response: text)
        }.sorted { $0.score > $1.score }
    }

    private func pretty(_ obj: Any) -> String {
        guard JSONSerialization.isValidJSONObject(obj), let data = try? JSONSerialization.data(withJSONObject: obj, options: [.prettyPrinted]), let s = String(data: data, encoding: .utf8) else { return "" }
        return s
    }

    private func callDefault(prompt: String) -> String {
        return PantherBridge.validateDefault(prompt: prompt)
    }
    private func callOpenAI(prompt: String, apiKey: String, model: String, base: String) -> String {
        return PantherBridge.validateOpenAI(prompt: prompt, apiKey: apiKey, model: model, base: base)
    }
    private func callOllama(prompt: String, base: String, model: String) -> String {
        return PantherBridge.validateOllama(prompt: prompt, base: base, model: model)
    }
    private func callMulti(prompt: String, providersJSON: String, withProof: Bool) -> String {
        if useCustomGuidelines, let data = customGuidelines.data(using: .utf8), (try? JSONSerialization.jsonObject(with: data)) is Any {
            return withProof ? PantherBridge.validateCustomWithProof(prompt: prompt, providersJSON: providersJSON, guidelinesJSON: customGuidelines)
                             : PantherBridge.validateCustom(prompt: prompt, providersJSON: providersJSON, guidelinesJSON: customGuidelines)
        }
        return withProof ? PantherBridge.validateMultiWithProof(prompt: prompt, providersJSON: providersJSON)
                         : PantherBridge.validateMulti(prompt: prompt, providersJSON: providersJSON)
    }

    // MARK: - Compliance helpers
    private func generateCompliance() {
        // bias over all responses
        let samples = results.map { $0.response }
        let s = PantherBridge.biasDetect(samples: samples)
        complianceReport = s
        // simple trust index heuristic: avg adherence penalized by bias
        let avgAdh = results.map{ $0.score/100.0 }.reduce(0,+) / Double(results.count)
        if let d = s.data(using: .utf8),
           let o = try? JSONSerialization.jsonObject(with: d) as? [String: Any],
           let bias = o["bias_score"] as? Double {
            trustIndex = max(0, min(1, avgAdh * (1.0 - bias)))
        } else {
            trustIndex = avgAdh
        }
    }

    private func loadGuidelinesFromURL() {
        PantherBridge.loadGuidelinesFromURL(guidelinesURL) { s in
            guard let s else { return }
            DispatchQueue.main.async {
                self.customGuidelines = s
                self.useCustomGuidelines = true
            }
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
                // pre-check OpenAI key presence when chosen
                if simMethod == .embedOpenAI {
                    let hasKey = ProcessInfo.processInfo.environment["PANTHER_OPENAI_API_KEY"].map { !$0.isEmpty } ?? false
                    if !hasKey { self.simNotice = "PANTHER_OPENAI_API_KEY ausente; usando BOW como fallback"; method = "bow" }
                }
                if method.starts(with: "embed-") { _ = PantherBridge.guidelinesBuildEmbeddings(method: method) }
            }
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
        let rc = PantherBridge.guidelinesSave(name: indexName, json: json)
        print("guidelinesSave rc=", rc)
    }

    private func loadIndex() {
        guard !indexName.trimmingCharacters(in: .whitespaces).isEmpty else { return }
        let n = PantherBridge.guidelinesLoad(name: indexName)
        print("guidelinesLoad n=", n)
        if n > 0 { fetchAndScore() }
    }

    private func extractCombinedHash() -> String? {
        guard let proof = proofJSON, let data = proof.data(using: .utf8) else { return nil }
        if let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any] {
            if let p = obj["proof"] as? [String: Any], let h = p["combined_hash"] as? String { return h }
            if let h = obj["combined_hash"] as? String { return h }
        }
        return nil
    }

    private func anchorProof() {
        guard let hash = extractCombinedHash() else { return }
        PantherBridge.anchorProof(apiBase: proofApiBase, hash: hash) { s in
            DispatchQueue.main.async { self.lastAnchorResponse = s }
        }
    }

    private func checkProofStatus() {
        guard let hash = extractCombinedHash() else { return }
        PantherBridge.checkProofStatus(apiBase: proofApiBase, hash: hash) { s in
            DispatchQueue.main.async { self.lastAnchorResponse = s }
        }
    }
}
