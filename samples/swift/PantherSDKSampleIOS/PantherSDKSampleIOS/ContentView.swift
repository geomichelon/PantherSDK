import SwiftUI

struct ContentView: View {
    enum Mode: String, CaseIterable { case single = "Single", multi = "Multi", proof = "With Proof" }
    enum Provider: String, CaseIterable { case openai = "OpenAI", ollama = "Ollama", anthropic = "Anthropic", `default` = "Default" }

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
                    Picker("Provider", selection: $provider) {
                        ForEach(Provider.allCases, id: \.self) { Text($0.rawValue).tag($0) }
                    }
                    .pickerStyle(.segmented)

                    Group {
                        if provider == .openai {
                            TextField("OpenAI API Key", text: $apiKey)
                                .textInputAutocapitalization(.never)
                                .disableAutocorrection(true)
                                .textFieldStyle(.roundedBorder)
                            TextField("Base URL", text: $openAIBase)
                                .textFieldStyle(.roundedBorder)
                            ModelPicker(title: "Model", presets: CostRules.openAIModels, selection: $openAIModel)
                        } else if provider == .ollama {
                            TextField("Ollama Base (http://127.0.0.1:11434)", text: $ollamaBase)
                                .textFieldStyle(.roundedBorder)
                            ModelPicker(title: "Model", presets: CostRules.ollamaModels, selection: $ollamaModel)
                        } else if provider == .anthropic {
                            TextField("Anthropic API Key", text: $anthropicKey)
                                .textInputAutocapitalization(.never)
                                .disableAutocorrection(true)
                                .textFieldStyle(.roundedBorder)
                            TextField("Base URL (https://api.anthropic.com)", text: $anthropicBase)
                                .textFieldStyle(.roundedBorder)
                            ModelPicker(title: "Model", presets: CostRules.anthropicModels, selection: $anthropicModel)
                        } else {
                            Text("Usando providers de ambiente (Default)")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                    }

                    SectionHeader("Diretrizes")
                    Toggle("Usar diretrizes customizadas (JSON)", isOn: $useCustomGuidelines)
                    if useCustomGuidelines {
                        TextEditor(text: $customGuidelines)
                            .frame(minHeight: 90)
                            .padding(8)
                            .background(Color(.secondarySystemBackground))
                            .cornerRadius(8)
                            .font(.caption)

                        Button("Preencher com exemplo") {
                            customGuidelines = getExampleGuidelines()
                        }
                        .buttonStyle(.bordered)
                        .padding(.top, 4)
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
                    }
                }
                .padding()
            }
            .navigationTitle("PantherSDK")
        }
        .sheet(isPresented: $showCostRules) {
            NavigationView {
                VStack(alignment: .leading) {
                    TextEditor(text: $costRulesJson)
                        .font(.system(.footnote, design: .monospaced))
                        .padding(8)
                        .background(Color(.secondarySystemBackground))
                        .cornerRadius(8)
                    HStack {
                        Button("Restaurar Padrão") { costRulesJson = CostRules.defaultJSON }
                        Spacer()
                        Button("Fechar") { showCostRules = false }
                            .buttonStyle(.borderedProminent)
                    }
                }
                .padding()
                .navigationTitle("Tabela de Custos")
            }
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
                switch provider {
                case .openai: output = callOpenAI(prompt: prompt, apiKey: apiKey, model: openAIModel, base: openAIBase)
                case .ollama: output = callOllama(prompt: prompt, base: ollamaBase, model: ollamaModel)
                case .anthropic:
                    // Use multi path with single entry
                    let arr: [[String: String]] = [["type": "anthropic", "api_key": anthropicKey, "model": anthropicModel, "base_url": anthropicBase]]
                    let data = try? JSONSerialization.data(withJSONObject: arr)
                    let singleJSON = String(data: data ?? Data("[]".utf8), encoding: .utf8) ?? "[]"
                    output = callMulti(prompt: prompt, providersJSON: singleJSON, withProof: false)
                case .default: output = callDefault(prompt: prompt)
                }
            case .multi:
                output = callMulti(prompt: prompt, providersJSON: buildProvidersJSON(), withProof: false)
            case .proof:
                output = callMulti(prompt: prompt, providersJSON: buildProvidersJSON(), withProof: true)
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
        arr.append(["type": "default", "model": "anvisa", "base_url": ""]) // comparação
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

    private func getExampleGuidelines() -> String {
        let exampleGuidelines = [
            [
                "topic": "Segurança em medicamentos",
                "expected_terms": [
                    "contraindicação",
                    "interação medicamentosa",
                    "efeitos adversos",
                    "posologia",
                    "advertências",
                    "cuidados especiais",
                    "monitoramento",
                    "orientação médica"
                ]
            ],
            [
                "topic": "Informações técnicas",
                "expected_terms": [
                    "princípio ativo",
                    "mecanismo de ação",
                    "farmacocinética",
                    "farmacodinâmica",
                    "biodisponibilidade",
                    "meia-vida",
                    "clearance",
                    "volume de distribuição"
                ]
            ],
            [
                "topic": "Aspectos regulatórios",
                "expected_terms": [
                    "ANVISA",
                    "registro",
                    "autorização",
                    "normas técnicas",
                    "farmacovigilância",
                    "estudos clínicos",
                    "evidências científicas",
                    "conformidade regulatória"
                ]
            ]
        ]

        if let jsonData = try? JSONSerialization.data(withJSONObject: exampleGuidelines, options: [.prettyPrinted]),
           let jsonString = String(data: jsonData, encoding: .utf8) {
            return jsonString
        }

        return "[]"
    }
}
