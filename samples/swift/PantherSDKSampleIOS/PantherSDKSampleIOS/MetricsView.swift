import SwiftUI

struct MetricsView: View {
    let rows: [ValidationRow]

    @State private var candidateIndex: Int = 0
    @State private var rougeRef: String = ""
    @State private var factsText: String = ""
    @State private var plagCorpus: String = ""
    @State private var plagNgram: Int = 3

    @State private var rougeL: Double? = nil
    @State private var factCoverage: Double? = nil
    @State private var factCheckAdv: Double? = nil
    @State private var plagScore: Double? = nil
    @State private var plagScoreN: Double? = nil

    var candidateText: String { rows.indices.contains(candidateIndex) ? rows[candidateIndex].response : "" }

    var body: some View {
        NavigationView {
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    SectionHeader("Seleção da resposta")
                    if rows.isEmpty {
                        Text("Nenhum resultado disponível. Rode uma validação primeiro.")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    } else {
                        Picker("Resposta", selection: $candidateIndex) {
                            ForEach(rows.indices, id: \.self) { idx in
                                Text("\(rows[idx].provider) – \(String(format: "%.1f%%", rows[idx].score))").tag(idx)
                            }
                        }
                        .pickerStyle(.wheel)
                        .frame(maxHeight: 140)
                        Button("Usar melhor por adherence") {
                            if let bestIdx = rows.enumerated().max(by: { $0.element.score < $1.element.score })?.offset { candidateIndex = bestIdx }
                        }.buttonStyle(.bordered)
                    }

                    // ROUGE-L
                    SectionHeader("ROUGE‑L")
                    Text("Referência (texto alvo)").font(.caption).foregroundColor(.secondary)
                    TextEditor(text: $rougeRef).frame(minHeight: 80).padding(8).background(Color(.secondarySystemBackground)).cornerRadius(8)
                    HStack {
                        Button("Calcular ROUGE‑L") { calcRougeL() }.buttonStyle(.borderedProminent)
                        if let r = rougeL { Text("ROUGE‑L: \(String(format: "%.3f", r))").font(.subheadline) }
                    }

                    // Fact coverage / fact-check
                    SectionHeader("Fact coverage / Fact‑check")
                    Text("Fatos (um por linha)").font(.caption).foregroundColor(.secondary)
                    TextEditor(text: $factsText).frame(minHeight: 80).padding(8).background(Color(.secondarySystemBackground)).cornerRadius(8)
                    HStack {
                        Button("Coverage") { calcFactCoverage() }.buttonStyle(.bordered)
                        if let fc = factCoverage { Text("\(String(format: "%.3f", fc))").font(.subheadline) }
                        Button("Fact‑check Adv") { calcFactCheckAdv() }.buttonStyle(.borderedProminent)
                        if let fa = factCheckAdv { Text("\(String(format: "%.3f", fa))").font(.subheadline) }
                    }

                    // Plágio
                    SectionHeader("Plágio")
                    Text("Corpus (uma amostra por linha)").font(.caption).foregroundColor(.secondary)
                    TextEditor(text: $plagCorpus).frame(minHeight: 80).padding(8).background(Color(.secondarySystemBackground)).cornerRadius(8)
                    HStack {
                        Button("Jaccard (default)") { calcPlag() }.buttonStyle(.bordered)
                        if let ps = plagScore { Text("\(String(format: "%.3f", ps))").font(.subheadline) }
                    }
                    HStack {
                        Stepper("n‑gram: \(plagNgram)", value: $plagNgram, in: 1...7)
                        Button("Jaccard n‑gram") { calcPlagN() }.buttonStyle(.borderedProminent)
                        if let pn = plagScoreN { Text("\(String(format: "%.3f", pn))").font(.subheadline) }
                    }
                }
                .padding()
            }
            .navigationTitle("Métricas")
            .onAppear {
                if let bestIdx = rows.enumerated().max(by: { $0.element.score < $1.element.score })?.offset { candidateIndex = bestIdx }
            }
        }
    }

    private func calcRougeL() {
        let ref = rougeRef.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !ref.isEmpty else { rougeL = nil; return }
        rougeL = PantherBridge.metricsROUGEL(reference: ref, candidate: candidateText)
    }
    private func calcFactCoverage() {
        let facts = factsText.split(separator: "\n").map { String($0).trimmingCharacters(in: .whitespacesAndNewlines) }.filter { !$0.isEmpty }
        guard !facts.isEmpty else { factCoverage = nil; return }
        factCoverage = PantherBridge.metricsFactCoverage(facts: facts, candidate: candidateText)
    }
    private func calcFactCheckAdv() {
        let facts = factsText.split(separator: "\n").map { String($0).trimmingCharacters(in: .whitespacesAndNewlines) }.filter { !$0.isEmpty }
        guard !facts.isEmpty else { factCheckAdv = nil; return }
        factCheckAdv = PantherBridge.metricsFactCheckAdv(facts: facts, candidate: candidateText)
    }
    private func calcPlag() {
        let corpus = plagCorpus.split(separator: "\n").map { String($0).trimmingCharacters(in: .whitespacesAndNewlines) }.filter { !$0.isEmpty }
        guard !corpus.isEmpty else { plagScore = nil; return }
        plagScore = PantherBridge.metricsPlagiarism(corpus: corpus, candidate: candidateText)
    }
    private func calcPlagN() {
        let corpus = plagCorpus.split(separator: "\n").map { String($0).trimmingCharacters(in: .whitespacesAndNewlines) }.filter { !$0.isEmpty }
        guard !corpus.isEmpty else { plagScoreN = nil; return }
        plagScoreN = PantherBridge.metricsPlagiarismNgram(corpus: corpus, candidate: candidateText, ngram: Int32(plagNgram))
    }
}

struct MetricsView_Previews: PreviewProvider {
    static var previews: some View {
        MetricsView(rows: [])
    }
}

