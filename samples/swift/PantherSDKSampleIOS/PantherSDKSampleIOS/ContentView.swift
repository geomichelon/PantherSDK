import SwiftUI

struct ContentView: View {
    enum Mode: String, CaseIterable { case single = "Single", multi = "Multi", proof = "With Proof" }
    enum Provider: String, CaseIterable { case openai = "OpenAI", ollama = "Ollama", anthropic = "Anthropic", `default` = "Default" }
<<<<<<< HEAD
    enum Strategy: String, CaseIterable { case baseline = "Baseline", structured = "Structured", comprehensive = "Full (JSON)", all = "Run All (3)" }
    @EnvironmentObject private var session: AppSession

    @State private var prompt: String = "Explain safe medication recommendations during pregnancy."
    @State private var mode: Mode = .single
    @State private var provider: Provider = .openai
    @State private var strategy: Strategy = .baseline
=======

    @State private var prompt: String = "Explique recomendações seguras de medicamentos na gravidez."
    @State private var mode: Mode = .single
    @State private var provider: Provider = .openai
>>>>>>> origin/main

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
<<<<<<< HEAD
    @FocusState private var guidelinesFocused: Bool
    private var isGuidelinesJSONValid: Bool {
        let s = customGuidelines.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !s.isEmpty, let data = s.data(using: .utf8) else { return false }
        return (try? JSONSerialization.jsonObject(with: data)) != nil
    }
=======
>>>>>>> origin/main

    @State private var showCostRules: Bool = false
    @State private var costRulesJson: String = CostRules.defaultJSON

    @State private var isRunning: Bool = false
    @State private var results: [ValidationRow] = []
    @State private var proofJSON: String? = nil

    // Compliance & Proof extras
    @State private var complianceReport: String = ""
    @State private var trustIndex: Double = 0
<<<<<<< HEAD
    @State private var trustIndexText: String = ""
    // Optional Prometheus Pushgateway export
    @State private var pushMetricsEnabled: Bool = false
    @State private var pushGatewayURL: String = "http://127.0.0.1:9091"
    @State private var guidelinesURL: String = ""
    @State private var proofApiBase: String = "http://127.0.0.1:8000"
    @State private var lastAnchorResponse: String = ""
    @State private var simRows: [SimilarityRow] = []
    enum SimMethod: String, CaseIterable { case bow = "BOW", jaccard = "Jaccard", hybrid = "Hybrid", embedOpenAI = "Embed(OpenAI)", embedOllama = "Embed(Ollama)" }
    @State private var simMethod: SimMethod = .bow
    @State private var indexName: String = "default"
    @State private var simNotice: String = ""
    @State private var showHelp: Bool = false
=======
    @State private var guidelinesURL: String = ""
    @State private var proofApiBase: String = "http://127.0.0.1:8000"
    @State private var lastAnchorResponse: String = ""
>>>>>>> origin/main

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

<<<<<<< HEAD
                    SectionHeader("Execution")
=======
                    SectionHeader("Execução")
>>>>>>> origin/main
                    Picker("Mode", selection: $mode) {
                        ForEach(Mode.allCases, id: \.self) { Text($0.rawValue).tag($0) }
                    }
                    .pickerStyle(.segmented)
<<<<<<< HEAD
                    Picker("Prompt Strategy", selection: $strategy) {
                        ForEach(Strategy.allCases, id: \.self) { Text($0.rawValue).tag($0) }
                    }
                    .pickerStyle(.segmented)
                    // Provider configuration moved to Providers tab (session-backed)

                    SectionHeader("Guidelines")
                    Toggle("Use custom guidelines (JSON)", isOn: $useCustomGuidelines)
                    if useCustomGuidelines {
                        TextEditor(text: $customGuidelines)
                            .focused($guidelinesFocused)
=======
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
>>>>>>> origin/main
                            .frame(minHeight: 90)
                            .padding(8)
                            .background(Color(.secondarySystemBackground))
                            .cornerRadius(8)
                            .font(.caption)
<<<<<<< HEAD
                        let trimmed = customGuidelines.trimmingCharacters(in: .whitespacesAndNewlines)
                        if trimmed.isEmpty {
                            Text("Empty JSON — using ANVISA (fallback)")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        } else if !isGuidelinesJSONValid {
                            Text("Invalid JSON — not valid JSON")
                                .font(.caption)
                                .foregroundColor(.red)
                        } else {
                            Text("Valid JSON")
                                .font(.caption)
                                .foregroundColor(.green)
                        }
                    } else {
                        Text("ANVISA (built-in default)")
=======
                    } else {
                        Text("ANVISA (padrão embutido)")
>>>>>>> origin/main
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }

                    HStack {
<<<<<<< HEAD
                        SectionHeader("Costs (estimate)")
                        Spacer()
                        Button("Edit Table") { showCostRules = true }
                            .buttonStyle(.borderedProminent)
                            .buttonBorderShape(.capsule)
=======
                        SectionHeader("Custos (estimativa)")
                        Spacer()
                        Button("Editar Tabela") { showCostRules = true }
                            .buttonStyle(.bordered)
>>>>>>> origin/main
                    }

                    Button(action: runValidation) {
                        HStack { isRunning ? AnyView(AnyView(ProgressView())) : AnyView(Image(systemName: "checkmark.shield")); Text("Validate") }
<<<<<<< HEAD
                        .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.borderedProminent)
                    .buttonBorderShape(.capsule)
                    .disabled(isRunning || prompt.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)

                    if !results.isEmpty {
                        SectionHeader("Results")
                        SummaryView(rows: results)
                        ForEach(results.indices, id: \.self) { i in ResultCard(row: $results[i]) }
                    }

                    if let proof = proofJSON, mode == .proof {
                        SectionHeader("Proof")
=======
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
>>>>>>> origin/main
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
<<<<<<< HEAD
                                Button("Anchor") { anchorProof() }
                                    .buttonStyle(.borderedProminent)
                                    .buttonBorderShape(.capsule)
                                Button("Status") { checkProofStatus() }
                                    .buttonStyle(.borderedProminent)
                                    .buttonBorderShape(.capsule)
=======
                                Button("Anchor") { anchorProof() }.buttonStyle(.borderedProminent)
                                Button("Status") { checkProofStatus() }
>>>>>>> origin/main
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
<<<<<<< HEAD
                        HStack { Button("Generate Compliance") { generateCompliance() }.buttonStyle(.borderedProminent).buttonBorderShape(.capsule); Spacer() }
                        // Optional Prometheus Pushgateway
                        Toggle("Push metrics to Prometheus (Pushgateway)", isOn: $pushMetricsEnabled)
                        if pushMetricsEnabled {
                            HStack {
                                TextField("http://127.0.0.1:9091", text: $pushGatewayURL)
                                    .textFieldStyle(.roundedBorder)
                                Button("Push now") { pushMetrics() }
                                    .buttonStyle(.borderedProminent).buttonBorderShape(.capsule)
                            }
                            Text("Sends trust_index, coverage_ratio and per-provider metrics to Pushgateway (job: panther, instance: ios)").font(.caption).foregroundColor(.secondary)
                        }
                        if !complianceReport.isEmpty {
                            Text("Trust Index: \(trustIndexText)")
                                .font(.subheadline)
                                .foregroundColor(trustIndex >= 0.8 ? .green : (trustIndex >= 0.6 ? .orange : .red))
                            HStack {
                                Text("Compliance JSON").font(.subheadline)
                                Spacer()
                                Button("Copy JSON") { UIPasteboard.general.string = complianceReport }
                                    .buttonStyle(.bordered)
                                    .buttonBorderShape(.capsule)
                            }
                            TextEditor(text: $complianceReport)
                                .font(.system(.footnote, design: .monospaced))
                                .frame(minHeight: 120)
=======
                        HStack { Button("Gerar Compliance") { generateCompliance() }; Spacer() }
                        if !complianceReport.isEmpty {
                            Text("Trust Index: \(String(format: "%.1f", trustIndex*100))%")
                                .font(.subheadline)
                                .foregroundColor(trustIndex >= 0.8 ? .green : (trustIndex >= 0.6 ? .orange : .red))
                            Text(complianceReport)
                                .font(.system(.footnote, design: .monospaced))
>>>>>>> origin/main
                                .padding(8)
                                .background(Color(.secondarySystemBackground))
                                .cornerRadius(8)
                        }
                    }

                    // Load Guidelines from URL helper
<<<<<<< HEAD
                    SectionHeader("Load Guidelines by URL")
                    VStack(alignment: .leading, spacing: 8) {
                        TextField("https://…/guidelines.json", text: $guidelinesURL)
                            .textFieldStyle(.roundedBorder)
                        HStack {
                            Button("Load") { loadGuidelinesFromURL() }
                                .buttonStyle(.borderedProminent)
                                .buttonBorderShape(.capsule)
                            Button("Fetch + scores") { fetchAndScore() }
                                .buttonStyle(.borderedProminent)
                                .buttonBorderShape(.capsule)
                        }
                    }

                    VStack(alignment: .leading, spacing: 8) {
                        TextField("Index name", text: $indexName)
                            .textFieldStyle(.roundedBorder)
                        HStack {
                            Button("Save index") { saveIndex() }
                                .buttonStyle(.borderedProminent)
                                .buttonBorderShape(.capsule)
                            Button("Load index") { loadIndex() }
                                .buttonStyle(.borderedProminent)
                                .buttonBorderShape(.capsule)
                        }
                    }

                    Picker("Method", selection: $simMethod) {
                        ForEach(SimMethod.allCases, id: \.self) { Text($0.rawValue).tag($0) }
                    }
                    .pickerStyle(.menu)

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
=======
                    SectionHeader("Carregar Diretrizes por URL")
                    HStack {
                        TextField("https://…/guidelines.json", text: $guidelinesURL)
                            .textFieldStyle(.roundedBorder)
                        Button("Carregar") { loadGuidelinesFromURL() }
>>>>>>> origin/main
                    }
                }
                .padding()
            }
<<<<<<< HEAD
            .simultaneousGesture(TapGesture().onEnded { guidelinesFocused = false })
            .hideKeyboardOnTap()
            .navigationTitle("Validate")
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button { showHelp = true } label: { Image(systemName: "questionmark.circle") }
                }
                ToolbarItemGroup(placement: .keyboard) {
                    Spacer()
                    Button("Close") { guidelinesFocused = false }
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
=======
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
>>>>>>> origin/main
        }
    }

    private func runValidation() {
        isRunning = true
        results = []
        proofJSON = nil
        DispatchQueue.global(qos: .userInitiated).async {
<<<<<<< HEAD
            func transformed(_ s: Strategy, _ p: String) -> (name: String, text: String) {
                switch s {
                case .baseline: return ("Baseline", p)
                case .structured:
                    let t = "Using best practices and clear structure, provide headings (General Recommendations, Risks, Dosing, Avoid, Vaccinations) and a short summary. Prompt: \(p)"
                    return ("Structured", t)
                case .comprehensive:
                    let t = "You are a healthcare compliance assistant. Output deterministic JSON with keys: summary, key_points[], risks[], recommendations[], contraindications[], checklist[]. Only output JSON. Prompt: \(p)"
                    return ("Full (JSON)", t)
                case .all: return ("", p) // handled externally
                }
            }

            func runOnce(prompt p: String) -> String {
                switch mode {
                case .single:
                    switch session.provider {
                    case .openai:
                        return callOpenAI(prompt: p, apiKey: session.openAIKey, model: session.openAIModel, base: session.openAIBase)
                    case .ollama:
                        return callOllama(prompt: p, base: session.ollamaBase, model: session.ollamaModel)
                    case .anthropic:
                        let arr: [[String: String]] = [["type": "anthropic", "api_key": session.anthropicKey, "model": session.anthropicModel, "base_url": session.anthropicBase]]
                        let data = try? JSONSerialization.data(withJSONObject: arr)
                        let singleJSON = String(data: data ?? Data("[]".utf8), encoding: .utf8) ?? "[]"
                        return callMulti(prompt: p, providersJSON: singleJSON, withProof: false)
                    case .default:
                        return callDefault(prompt: p)
                    }
                case .multi:
                    return callMulti(prompt: p, providersJSON: session.providersJSON(), withProof: false)
                case .proof:
                    return callMulti(prompt: p, providersJSON: session.providersJSON(), withProof: true)
                }
            }

            var allRows: [ValidationRow] = []
            if strategy == .all {
                for s in [Strategy.baseline, .structured, .comprehensive] {
                    let (name, text) = transformed(s, prompt)
                    let out = runOnce(prompt: text)
                    allRows.append(contentsOf: rowsFromOutput(out, strategy: name))
                }
            } else {
                let (name, text) = transformed(strategy, prompt)
                let out = runOnce(prompt: text)
                allRows.append(contentsOf: rowsFromOutput(out, strategy: name))
            }
            // Custom sort: group by provider type, then by score desc
            func groupOrder(_ provider: String) -> Int {
                if provider.hasPrefix("openai:") { return 0 }
                if provider.hasPrefix("anthropic:") { return 1 }
                if provider.hasPrefix("ollama:") { return 2 }
                return 3
            }
            let sorted = allRows.sorted { a, b in
                let ga = groupOrder(a.provider), gb = groupOrder(b.provider)
                if ga != gb { return ga < gb }
                return a.score > b.score
            }
            DispatchQueue.main.async {
                isRunning = false
                results = sorted
=======
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
>>>>>>> origin/main
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
<<<<<<< HEAD
=======
        arr.append(["type": "default", "model": "anvisa", "base_url": ""]) // comparação
>>>>>>> origin/main
        let data = try? JSONSerialization.data(withJSONObject: arr)
        return String(data: data ?? Data("[]".utf8), encoding: .utf8) ?? "[]"
    }

<<<<<<< HEAD
    private func rowsFromOutput(_ s: String, strategy: String?) -> [ValidationRow] {
=======
    private func parseAndDisplay(_ s: String) {
>>>>>>> origin/main
        if mode == .proof, let data = s.data(using: .utf8),
           let obj = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any],
           let arr = obj["results"], let arrData = try? JSONSerialization.data(withJSONObject: arr),
           let json = try? JSONSerialization.jsonObject(with: arrData) as? [[String: Any]] {
            self.proofJSON = pretty(obj)
<<<<<<< HEAD
            return rows(from: json, strategy: strategy)
        }
        if let data = s.data(using: .utf8), let json = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]] {
            return rows(from: json, strategy: strategy)
        }
        return []
    }

    private func rows(from arr: [[String: Any]], strategy: String?) -> [ValidationRow] {
=======
            self.results = rows(from: json)
            return
        }
        if let data = s.data(using: .utf8), let json = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]] {
            self.results = rows(from: json)
        }
    }

    private func rows(from arr: [[String: Any]]) -> [ValidationRow] {
>>>>>>> origin/main
        let tokensIn = PantherBridge.tokenCount(prompt)
        let rules = costRulesJson
        return arr.compactMap { r -> ValidationRow? in
            let name = r["provider_name"] as? String ?? "?"
            let score = r["adherence_score"] as? Double ?? 0
            let lat = r["latency_ms"] as? Int ?? 0
            let text = r["raw_text"] as? String ?? ""
            let tokensOut = PantherBridge.tokenCount(text)
            let cost = PantherBridge.calculateCost(tokensIn: tokensIn, tokensOut: tokensOut, provider: name, rules: rules)
<<<<<<< HEAD
            let coh = PantherBridge.coherence(text)
            let flu = PantherBridge.fluency(text)
            return ValidationRow(provider: name, score: score, latencyMs: lat, tokensIn: tokensIn, tokensOut: tokensOut, costUSD: cost, response: text, strategy: strategy, coherence: coh, fluency: flu)
        }
=======
            return ValidationRow(provider: name, score: score, latencyMs: lat, tokensIn: tokensIn, tokensOut: tokensOut, costUSD: cost, response: text)
        }.sorted { $0.score > $1.score }
>>>>>>> origin/main
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
<<<<<<< HEAD
        // Consider only successful rows (score > 0) to avoid upstream error rows skewing trust
        let ok = results.filter { $0.score > 0 }
        guard !ok.isEmpty else {
            complianceReport = "{\"notice\":\"no valid results to compute compliance\"}"
            trustIndex = 0
            return
        }

        // Domain‑aligned safety coverage check (simple substring heuristics over all responses)
        // This replaces the previous pronoun-based bias metric.
        let text = ok.map { $0.response.lowercased() }.joined(separator: "\n")
        func hasAny(_ keys: [String]) -> Bool { keys.contains { text.contains($0) } }

        // Tighten keywords to reduce false positives (removed very generic tokens like "risk", "dose", "vaccine")
        let coverage: [String: Bool] = [
            "consultation": hasAny(["consult healthcare provider", "consult a doctor", "consult your doctor", "professional guidance", "medical consultation"]),
            "risk_caution": hasAny(["contraindicated", "avoid ", " do not ", "not safe", "third trimester", "tetracycline", "isotretinoin", "nsaid", "ibuprofen"]),
            "dose_minimum": hasAny(["lowest effective dose", "shortest duration"]),
            "prenatal_vitamins": hasAny(["prenatal vitamin", "prenatal vitamins", "folic acid", "folate"]),
            "vaccinations": hasAny(["flu shot", "tdap"]),
            "labeling_info": hasAny(["fda pregnancy", "pllr", "pregnancy and lactation labeling"]) 
        ]
        let present = coverage.values.filter { $0 }.count
        let total = coverage.count
        let coverageRatio = total > 0 ? Double(present) / Double(total) : 0.0

        // Trust Index: with a single valid row, match adherence; otherwise scale by coverage (conservative)
        let avgAdh = ok.map{ $0.score/100.0 }.reduce(0,+) / Double(ok.count)
        if results.count == 1 {
            trustIndex = max(0, min(1, avgAdh))
        } else {
            trustIndex = max(0, min(1, avgAdh * coverageRatio))
        }
        trustIndexText = String(format: "%.1f%%", trustIndex*100)

        if pushMetricsEnabled { pushMetrics(coverageRatio: coverageRatio) }

        // Build a compact JSON report: coverage booleans + ratio + missing categories
        let missing = coverage.filter { !$0.value }.map { $0.key }
        let report: [String: Any] = [
            "safety_coverage": coverage,
            "coverage_ratio": coverageRatio,
            "missing_categories": missing
        ]
        if let data = try? JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted]),
           let s = String(data: data, encoding: .utf8) {
            complianceReport = s
        } else {
            complianceReport = "{}"
        }
    }

    private func pushMetrics(coverageRatio: Double? = nil) {
        guard pushMetricsEnabled, let url = URL(string: pushGatewayURL.trimmingCharacters(in: .whitespacesAndNewlines)) else { return }
        // Build Prometheus exposition format
        var lines: [String] = []
        // run-level
        lines.append(String(format: "panther_trust_index %.6f", trustIndex))
        if let cov = coverageRatio ?? extractCoverageRatio() {
            lines.append(String(format: "panther_coverage_ratio %.6f", cov))
        }
        // per-provider
        for r in results where r.score > 0 {
            let prov = r.provider.replacingOccurrences(of: "\"", with: "'")
            lines.append(String(format: "panther_adherence_score{provider=\"%@\"} %.6f", prov, r.score/100.0))
            lines.append(String(format: "panther_latency_ms{provider=\"%@\"} %d", prov, r.latencyMs))
            lines.append(String(format: "panther_cost_usd{provider=\"%@\"} %.6f", prov, r.costUSD))
        }
        let body = lines.joined(separator: "\n") + "\n"
        // Pushgateway path: /metrics/job/<job>/instance/<instance>
        var pushURL = url
        if pushURL.path.isEmpty || pushURL.path == "/" {
            pushURL.append(path: "/metrics/job/panther/instance/ios")
        }
        var req = URLRequest(url: pushURL)
        req.httpMethod = "POST"
        req.setValue("text/plain; version=0.0.4", forHTTPHeaderField: "Content-Type")
        req.httpBody = body.data(using: .utf8)
        URLSession.shared.dataTask(with: req) { _, _, _ in }.resume()
    }

    private func extractCoverageRatio() -> Double? {
        guard let data = complianceReport.data(using: .utf8),
              let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let cov = obj["coverage_ratio"] as? Double else { return nil }
        return cov
=======
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
>>>>>>> origin/main
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

<<<<<<< HEAD
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

=======
>>>>>>> origin/main
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
