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
    @State private var showResetAlert: Bool = false
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
                    Section { Text("Using environment providers (Default)").font(.caption).foregroundColor(.secondary) }
                }

                // Advanced: allow multi-provider JSON override used by Validate -> Multi/With Proof
                Section(header: Text("LLMs (Advanced JSON)")) {
                    Toggle("Use LLMs (advanced JSON)", isOn: $session.useAdvancedProvidersJSON)
                    if session.useAdvancedProvidersJSON {
                        TextEditor(text: $session.advancedProvidersJSON)
                            .focused($advProvidersFocused)
                            .font(.system(.footnote, design: .monospaced))
                            .frame(minHeight: 120)
                            .padding(6)
                            .background(Color(UIColor.secondarySystemBackground))
                            .cornerRadius(8)
                        if advIsEmpty {
                            Text("Empty JSON — Multi will use Single LLM (fallback)")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        } else if !advIsValid {
                            Text("Invalid JSON — not valid JSON")
                                .font(.caption)
                                .foregroundColor(.red)
                        } else {
                            Text("Valid JSON")
                                .font(.caption)
                                .foregroundColor(.green)
                        }
                        VStack(alignment: .leading, spacing: 8) {
                            Button {
                                // Ready-made example: OpenAI (all preset models)
                                let base = session.openAIBase.isEmpty ? "https://api.openai.com" : session.openAIBase
                                let key = session.openAIKey
                                var arr: [[String: String]] = []
                                for mdl in CostRules.openAIModels {
                                    arr.append(["type": "openai", "api_key": key, "base_url": base, "model": mdl])
                                }
                                if let data = try? JSONSerialization.data(withJSONObject: arr, options: [.prettyPrinted]), let s = String(data: data, encoding: .utf8) {
                                    session.useAdvancedProvidersJSON = true
                                    session.advancedProvidersJSON = s
                                    session.save()
                                    jsonAlertText = "OpenAI models example filled. Advanced JSON enabled."
                                    showJsonAlert = true
                                }
                            } label: {
                                Label("Fill Example (OpenAI — all presets)", systemImage: "wand.and.stars")
                                    .frame(maxWidth: .infinity)
                            }
                            .buttonStyle(.borderedProminent)

                            Button {
                                _ = session.save()
                                jsonAlertText = "JSON saved"
                                showJsonAlert = true
                            } label: {
                                Label("Save JSON", systemImage: "square.and.arrow.down")
                            }
                            .buttonStyle(.borderedProminent)
                            .buttonBorderShape(.capsule)
                        }
                        Text("The JSON above will be used in Multi/With Proof. Turn off to use the single selection on this screen.")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }
                Section(header: Text("Costs")) {
                    Button("Edit Table") { showCostRules = true }
                        .buttonStyle(.borderedProminent)
                        .buttonBorderShape(.capsule)
                }
                Section {
                    Button("Save Session") { session.save() }
                        .buttonStyle(.borderedProminent)
                        .buttonBorderShape(.capsule)
                    Button(role: .destructive) { showResetAlert = true } label: { Text("Reset Session") }
                        .buttonStyle(.borderedProminent)
                        .buttonBorderShape(.capsule)
                }
            }
            .simultaneousGesture(TapGesture().onEnded { advProvidersFocused = false })
            .hideKeyboardOnTap()
            .navigationTitle("LLMs")
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button { showHelp = true } label: { Image(systemName: "questionmark.circle") }
                }
                ToolbarItemGroup(placement: .keyboard) {
                    Spacer()
                Button("Close") { advProvidersFocused = false }
                    .buttonStyle(.borderedProminent)
                    .buttonBorderShape(.capsule)
                }
            }
            .alert(jsonAlertText, isPresented: $showJsonAlert) { Button("OK", role: .cancel) {} }
            .alert("Reset session?", isPresented: $showResetAlert) {
                Button("Cancel", role: .cancel) {}
                Button("Reset", role: .destructive) { session.reset() }
            } message: {
                Text("This will clear saved API keys, URLs, models and the advanced JSON.")
            }
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
                Button("Restore Default") { costRulesJson = CostRules.defaultJSON }
                Spacer()
                Button("Close") { dismiss() }.buttonStyle(.borderedProminent)
            }
        }
        .padding()
        .navigationTitle("Cost Table")
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
                SectionHeader("Prompt for similarity")
                    TextEditor(text: $prompt)
                        .frame(minHeight: 70)
                        .padding(8)
                        .background(Color(UIColor.secondarySystemBackground))
                        .cornerRadius(8)
                    SectionHeader("Guidelines")
                    Toggle("Usar diretrizes customizadas (JSON)", isOn: $useCustomGuidelines)
                    if useCustomGuidelines {
                        TextEditor(text: $customGuidelines)
                            .frame(minHeight: 90)
                            .padding(8)
                            .background(Color(UIColor.secondarySystemBackground))
                            .cornerRadius(8)
                            .font(.caption)
                    } else {
                        Text("ANVISA (built-in default)").font(.caption).foregroundColor(.secondary)
                    }
                    HStack {
                        TextField("https://…/guidelines.json", text: $guidelinesURL).textFieldStyle(.roundedBorder)
                        Button("Load") { loadGuidelinesFromURL() }
                        Button("Fetch + scores") { fetchAndScore() }
                    }
                    HStack {
                        TextField("Index name", text: $indexName).textFieldStyle(.roundedBorder)
                        Button("Save index") { saveIndex() }
                        Button("Load index") { loadIndex() }
                    }
                    Picker("Method", selection: $simMethod) {
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
        .hideKeyboardOnTap()
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
        case .bow: return "BOW: Bag‑of‑Words with cosine (local/fast)."
        case .jaccard: return "Jaccard: set similarity of tokens (local/fast)."
        case .hybrid: return "Hybrid: BOW + Jaccard (balanced precision/recall)."
        case .embedOpenAI: return "Embeddings (OpenAI): vector similarity; requires PANTHER_OPENAI_API_KEY."
        case .embedOllama: return "Embeddings (Ollama): local vector similarity; requires PANTHER_OLLAMA_BASE."
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
                Text("Configure the 'Single' LLM and use 'LLMs (Advanced JSON)' for Multi/Proof.")
                    .font(.subheadline)
                SectionHeader("LLMs (Advanced JSON)")
                Text("Example lists for Multi/Proof:")
                    .font(.subheadline)
                Text("OpenAI — gpt-4o + gpt-4.1-mini").font(.caption).foregroundColor(.secondary)
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
                SectionHeader("Validate — Modes")
                Text("Single — Runs the provider/model selected in the Providers tab.")
                    .font(.subheadline)
                Text("Multi — Runs all providers from ‘LLMs (Advanced JSON)’ in parallel (falls back to Single if empty).")
                    .font(.subheadline)
                Text("With Proof — Same as Multi, plus a {results, proof} bundle with a combined audit hash used by Anchor/Status.")
                    .font(.subheadline)

                SectionHeader("Prompt Strategy")
                Text("Baseline — Sends the prompt as-is. Lowest cost/latency; free-form output; adherence/coverage depend on your prompt.")
                    .font(.subheadline)
                Text("Structured — Adds best-practice instructions and requests sections (General Recommendations, Risks, Dosing, Avoid, Vaccinations) plus a short summary. Tends to improve adherence and safety coverage with moderate cost.")
                    .font(.subheadline)
                Text("Full (JSON) — Deterministic JSON with keys {summary, key_points[], risks[], recommendations[], contraindications[], checklist[]}. Maximum predictability and easy parsing; higher token cost.")
                    .font(.subheadline)
                Text("Run All (3) — Runs Baseline, Structured and Full (JSON) and labels each card with its strategy for side-by-side comparison.")
                    .font(.subheadline)
                SectionHeader("Guidelines")
                Text("Without JSON: uses built-in ANVISA. With JSON: array {topic, expected_terms[]}. Adherence checks presence of expected terms.")
                    .font(.subheadline)
                SectionHeader("Similarity")
                Text("BOW/Jaccard/Hybrid are local; Embeddings (OpenAI/Ollama) require credentials/endpoint.")
                    .font(.subheadline)
                SectionHeader("Costs")
                Text("Price table per 1k tokens; used to estimate cost in Validate.")
                    .font(.subheadline)
            }
            .padding()
        }
        .navigationTitle("Help")
    }
}
