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

    @State private var customGuidelines: String = ""

    @State private var showCostRules: Bool = false
    @State private var costRulesJson: String = CostRules.defaultJSON

    @State private var isRunning: Bool = false
    @State private var results: [ValidationRow] = []
    @State private var proofJSON: String? = nil

    // Compliance & Proof extras
    @State private var complianceReport: String = ""
    @State private var trustIndex: Double = 0
    @State private var includeFailedModels: Bool = false
    @State private var guidelinesURL: String = ""
    @State private var proofApiBase: String = "http://127.0.0.1:8000"
    @State private var lastAnchorResponse: String = ""
    // Neutral reference used for BLEU comparison (content validation)
    @State private var neutralReference: String = "O uso de medicamentos na gravidez deve ser baseado em avaliação risco‑benefício individualizada por profissional habilitado. Priorizar fármacos com histórico de segurança e categoria de risco favorável, evitando agentes potencialmente teratogênicos. Considerar o princípio ativo e seu mecanismo de ação, bem como alterações farmacocinéticas da gestação para ajuste de posologia. Verificar contraindicações, advertências, interações medicamentosas e possíveis efeitos adversos, com monitoramento clínico adequado. Seguir as normas e orientações da ANVISA e da literatura atualizada. Orientar a paciente a não realizar automedicação e a buscar acompanhamento médico para qualquer dúvida ou sintoma."
    @State private var bleuWeight: Double = 0.3
    @State private var bleuThreshold: Double = 70.0
    @State private var showHelper: Bool = false
    @State private var showMetrics: Bool = false

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

                    if mode == .single {
                        Picker("Provider", selection: $provider) {
                            ForEach(Provider.allCases, id: \.self) { Text($0.rawValue).tag($0) }
                        }
                        .pickerStyle(.segmented)
                    } else {
                        SectionHeader("Providers a serem testados")
                        let activeProviders = getActiveProviders()
                        if activeProviders.isEmpty {
                            Text("Configure pelo menos uma chave API para testar")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        } else {
                            ForEach(activeProviders, id: \.self) { provider in
                                HStack {
                                    Image(systemName: "checkmark.circle.fill")
                                        .foregroundColor(.green)
                                    Text(provider)
                                        .font(.caption)
                                }
                            }
                        }
                    }

                    if mode == .single {
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
                    }

                    // Diretrizes: carregue por URL na seção abaixo

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
                        ForEach(results) { row in ResultCard(row: row, bleuWeight: bleuWeight, bleuThreshold: bleuThreshold) }
                    }

                    if let proof = proofJSON, mode == .proof {
                        SectionHeader("Prova")
                        Text(proof)
                            .font(.system(.footnote, design: .monospaced))
                            .padding(8)
                            .background(Color(.secondarySystemBackground))
                            .cornerRadius(8)

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

                    if !results.isEmpty {
                        SectionHeader("Compliance Report")
                        HStack {
                            Button("Gerar Compliance") { generateCompliance() }
                            Spacer()
                            Toggle("Incluir modelos com erro", isOn: $includeFailedModels)
                                .toggleStyle(.switch)
                                .font(.caption)
                        }
                        Text("Neutral Reference (BLEU)").font(.caption).foregroundColor(.secondary)
                        TextEditor(text: $neutralReference)
                            .frame(minHeight: 80)
                            .padding(8)
                            .background(Color(.secondarySystemBackground))
                            .cornerRadius(8)

                        Text("Dica: separe múltiplas referências por linha em branco — usamos o melhor BLEU.")
                            .font(.caption2)
                            .foregroundColor(.secondary)
                        HStack {
                            VStack(alignment: .leading) {
                                Text("Peso BLEU: \(Int(bleuWeight*100))%")
                                    .font(.caption)
                                Slider(value: $bleuWeight, in: 0...1, step: 0.05)
                            }
                            VStack(alignment: .leading) {
                                Text("Limiar BLEU: \(Int(bleuThreshold))")
                                    .font(.caption)
                                Slider(value: $bleuThreshold, in: 0...100, step: 5)
                            }
                        }

                        if !complianceReport.isEmpty {
                            let successfulModels = results.filter { $0.score > 0 }
                            let failedModels = results.filter { $0.score == 0 }

                            HStack {
                                Text("✅ \(successfulModels.count) modelos funcionaram")
                                    .font(.caption)
                                    .foregroundColor(.green)
                                Spacer()
                                Text("❌ \(failedModels.count) modelos com erro")
                                    .font(.caption)
                                    .foregroundColor(.red)
                            }
                            .padding(.vertical, 4)

                            Text("Compliance (Adherence média): \(String(format: "%.1f", trustIndex*100))%")
                                .font(.subheadline)
                                .foregroundColor(trustIndex >= 0.8 ? .green : (trustIndex >= 0.6 ? .orange : .red))

                            let filtered = includeFailedModels ? results : results.filter { $0.score > 0 }
                            let avgQuality: Double = {
                                let vals = filtered.map { (1.0 - bleuWeight) * $0.score + bleuWeight * ($0.bleu ?? 0.0) }
                                return vals.isEmpty ? 0.0 : (vals.reduce(0,+) / Double(vals.count))
                            }()
                            Text("Qualidade (Adh + BLEU): \(String(format: "%.1f", avgQuality))%")
                                .font(.subheadline)
                                .foregroundColor(avgQuality >= 80 ? .green : (avgQuality >= 60 ? .orange : .red))

                            TextEditor(text: $complianceReport)
                                .font(.system(.footnote, design: .monospaced))
                                .frame(minHeight: 120)
                                .padding(8)
                                .background(Color(.secondarySystemBackground))
                                .cornerRadius(8)
                        }
                    }

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
            .toolbar {
                ToolbarItemGroup(placement: .navigationBarTrailing) {
                    Button {
                        showMetrics = true
                    } label: {
                        Image(systemName: "chart.bar")
                    }
                    .accessibilityLabel("Métricas")
                    Button {
                        showHelper = true
                    } label: {
                        Image(systemName: "questionmark.circle")
                    }
                    .accessibilityLabel("Ajuda")
                }
            }
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
        .sheet(isPresented: $showHelper) { HelperView() }
        .sheet(isPresented: $showMetrics) { MetricsView(rows: results) }
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

        if mode == .multi {
            if !apiKey.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                for model in CostRules.openAIModels {
                    arr.append(["type": "openai", "api_key": apiKey, "model": model, "base_url": openAIBase])
                }
            }

            for model in CostRules.ollamaModels {
                arr.append(["type": "ollama", "model": model, "base_url": ollamaBase])
            }

            if !anthropicKey.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                for model in CostRules.anthropicModels {
                    arr.append(["type": "anthropic", "api_key": anthropicKey, "model": model, "base_url": anthropicBase])
                }
            }
        } else {
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
        }

        // Não incluir diretrizes padrão (ANVISA). Use apenas quando customGuidelines for carregado.

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
            // Compute BLEU vs neutral reference(s) if provided
            let ref = neutralReference.trimmingCharacters(in: .whitespacesAndNewlines)
            let bleuPct: Double? = ref.isEmpty ? nil : computeBLEUPercent(refsText: ref, candidate: text)
            return ValidationRow(provider: name, score: score, bleu: bleuPct, latencyMs: lat, tokensIn: tokensIn, tokensOut: tokensOut, costUSD: cost, response: text)
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
        if !customGuidelines.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
           let data = customGuidelines.data(using: .utf8), (try? JSONSerialization.jsonObject(with: data)) is Any {
            return withProof ? PantherBridge.validateCustomWithProof(prompt: prompt, providersJSON: providersJSON, guidelinesJSON: customGuidelines)
                             : PantherBridge.validateCustom(prompt: prompt, providersJSON: providersJSON, guidelinesJSON: customGuidelines)
        }
        return withProof ? PantherBridge.validateMultiWithProof(prompt: prompt, providersJSON: providersJSON)
                         : PantherBridge.validateMulti(prompt: prompt, providersJSON: providersJSON)
    }

    private func generateCompliance() {
        let filteredResults = includeFailedModels ? results : results.filter { $0.score > 0 }

        if filteredResults.isEmpty {
            complianceReport = "Nenhum modelo válido para análise de compliance."
            trustIndex = 0.0
            return
        }

        // Recompute BLEU for current neutral reference (if provided)
        let ref = neutralReference.trimmingCharacters(in: .whitespacesAndNewlines)
        if !ref.isEmpty {
            results = results.map { row in
                let b = computeBLEUPercent(refsText: ref, candidate: row.response)
                return ValidationRow(provider: row.provider, score: row.score, bleu: b, latencyMs: row.latencyMs, tokensIn: row.tokensIn, tokensOut: row.tokensOut, costUSD: row.costUSD, response: row.response)
            }
        } else if let best = results.max(by: { $0.score < $1.score }) {
            // Fallback: Relative BLEU using the best adherence response as reference
            let autoRef = best.response
            results = results.map { row in
                let b = computeBLEUPercent(refsText: autoRef, candidate: row.response)
                return ValidationRow(provider: row.provider, score: row.score, bleu: b, latencyMs: row.latencyMs, tokensIn: row.tokensIn, tokensOut: row.tokensOut, costUSD: row.costUSD, response: row.response)
            }
        }

        // Focus on adherence metrics in the textual report

        var report = "=== ANÁLISE DE COMPLIANCE ===\n\n"
        report += "📊 RESUMO DOS MODELOS:\n"
        report += "• Total: \(results.count)\n"
        report += "• ✅ Funcionais: \(filteredResults.count)\n"
        report += "• ❌ Com erro: \(results.count - filteredResults.count)\n\n"

        report += "📋 ANÁLISE DE TERMOS:\n\n"

        if !customGuidelines.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
           let data = customGuidelines.data(using: .utf8),
           let array = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]] {

            for guideline in array {
                if let topic = guideline["topic"] as? String,
                   let expectedTerms = guideline["expected_terms"] as? [String] {

                    report += "📌 \(topic) (\(expectedTerms.count) termos):\n"
                    report += "   \(expectedTerms.joined(separator: ", "))\n"

                    for result in filteredResults {
                        let responseLower = result.response.lowercased()
                        let termsFound = expectedTerms.filter { responseLower.contains($0.lowercased()) }
                        let status = !termsFound.isEmpty ? "✅" : "❌"
                        report += "   \(status) \(result.provider)\n"
                    }
                    report += "\n"
                }
            }
        } else {
            // Sem diretrizes: derive termos da Neutral Reference (se disponível)
            let ref = neutralReference.trimmingCharacters(in: .whitespacesAndNewlines)
            let derived = deriveKeyTerms(from: ref, maxTerms: 10)
            if !derived.isEmpty {
                report += "(Sem diretriz por URL) – Usando termos derivados da referência neutra.\n\n"
                report += "📌 Termos (\(derived.count)):\n"
                report += "   \(derived.joined(separator: ", "))\n"
                for result in filteredResults {
                    let responseLower = result.response.lowercased()
                    let covered = derived.filter { responseLower.contains($0.lowercased()) }
                    let status = covered.isEmpty ? "❌" : "✅"
                    let covTxt = covered.isEmpty ? "(nenhum)" : covered.joined(separator: ", ")
                    report += "   \(status) \(result.provider) – cobre: \(covTxt)\n"
                }
                // Ranking por cobertura de termos
                let ranked = filteredResults.map { r -> (String, Int) in
                    let low = r.response.lowercased()
                    let c = derived.filter { low.contains($0.lowercased()) }.count
                    return (r.provider, c)
                }.sorted { $0.1 > $1.1 }
                if !ranked.isEmpty {
                    report += "\n🏆 Cobertura por termos (Top):\n"
                    for (i, item) in ranked.prefix(3).enumerated() {
                        report += "   \(i+1). \(item.0) – \(item.1)/\(derived.count) termos\n"
                    }
                }
                report += "\n"
            } else {
                report += "(Nenhuma diretriz carregada)\n"
                report += "Use \"Carregar Diretrizes por URL\" para habilitar a checagem de termos.\n\n"
            }
        }

        // BLEU is displayed at the result level; report focuses on adherence terms.

        complianceReport = report

        let avgAdh = filteredResults.map{ $0.score/100.0 }.reduce(0,+) / Double(filteredResults.count)
        trustIndex = avgAdh
    }

    private func computeBLEUPercent(refsText: String, candidate: String) -> Double {
        let norm = refsText.replacingOccurrences(of: "\r\n", with: "\n")
        let blocks = norm.components(separatedBy: "\n\n").map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }.filter { !$0.isEmpty }
        if blocks.isEmpty {
            let v = PantherBridge.metricsBLEU(reference: norm, candidate: candidate)
            return v > 1.0 ? min(v, 100.0) : (v * 100.0)
        }
        var best: Double = 0.0
        for ref in blocks {
            let v = PantherBridge.metricsBLEU(reference: ref, candidate: candidate)
            let pct = v > 1.0 ? min(v, 100.0) : (v * 100.0)
            if pct > best { best = pct }
        }
        return best
    }

    // Deriva termos-chave de uma referência neutra quando não há diretrizes carregadas
    private func deriveKeyTerms(from ref: String, maxTerms: Int) -> [String] {
        let trimmed = ref.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return [] }
        let stop: Set<String> = [
            "a","o","as","os","de","da","do","das","dos","e","em","para","por","com","uma","um","na","no","nas","nos","ou","que","se","ao","à","às","aos","entre","sobre","como","pela","pelo","pelas","pelos","mais","menos","contra","pode","devem","deve","ser","são","é","foi","sua","seu","suas","seus","quando","onde","qual","quais","também","bem","há"
        ]
        let tokens = trimmed
            .lowercased()
            .components(separatedBy: CharacterSet.alphanumerics.inverted)
            .filter { $0.count >= 5 && !stop.contains($0) }
        var freq: [String:Int] = [:]
        for t in tokens { freq[t, default: 0] += 1 }
        let sorted = freq.keys.sorted { (freq[$0] ?? 0) > (freq[$1] ?? 0) }
        return Array(sorted.prefix(maxTerms))
    }

    // Removed bias score extraction; sample now focuses on content adherence only

    private func loadGuidelinesFromURL() {
        PantherBridge.loadGuidelinesFromURL(guidelinesURL) { s in
            guard let s else { return }
            DispatchQueue.main.async {
                self.customGuidelines = s
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

    // Removido exemplo embutido de diretrizes; usar carregamento por URL

    private func getActiveProviders() -> [String] {
        var providers: [String] = []

        if mode == .multi {
            if !apiKey.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                for model in CostRules.openAIModels {
                    providers.append("OpenAI (\(model))")
                }
            }

            for model in CostRules.ollamaModels {
                providers.append("Ollama (\(model))")
            }

            if !anthropicKey.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                for model in CostRules.anthropicModels {
                    providers.append("Anthropic (\(model))")
                }
            }
        } else {
            switch provider {
            case .openai:
                if !apiKey.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                    providers.append("OpenAI (\(openAIModel))")
                }
            case .ollama:
                providers.append("Ollama (\(ollamaModel))")
            case .anthropic:
                if !anthropicKey.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                    providers.append("Anthropic (\(anthropicModel))")
                }
            case .default:
                break
            }
        }

        return providers
    }
}
