import SwiftUI

struct SectionHeader: View {
    let title: String
    init(_ t: String) { title = t }
    var body: some View { Text(title).font(.headline) }
}

struct ModelPicker: View {
    let title: String
    let presets: [String]
    @Binding var selection: String
    var body: some View {
        VStack(alignment: .leading) {
            Text(title).font(.subheadline)
            Picker(title, selection: $selection) { ForEach(presets, id: \.self) { Text($0).tag($0) } }
                .pickerStyle(.menu)
            TextField("Custom", text: $selection).textFieldStyle(.roundedBorder)
        }
    }
}

struct SummaryView: View {
    let rows: [ValidationRow]
    var best: ValidationRow? { rows.max(by: { $0.score < $1.score }) }
    var avgLatency: Int { guard !rows.isEmpty else { return 0 }; return rows.map { $0.latencyMs }.reduce(0, +) / rows.count }
    var avgBleu: Double? {
        let vals = rows.compactMap { $0.bleu }
        guard !vals.isEmpty else { return nil }
        return vals.reduce(0,+) / Double(vals.count)
    }
    var body: some View {
        HStack {
            VStack(alignment: .leading) { Text("Melhor provedor").font(.caption).foregroundColor(.secondary); Text(best?.provider ?? "–").font(.headline) }
            Spacer()
            VStack(alignment: .trailing) { Text("Latência média").font(.caption).foregroundColor(.secondary); Text("\(avgLatency) ms").font(.headline) }
            Spacer()
            if let b = avgBleu {
                Spacer()
                VStack(alignment: .trailing) { Text("BLEU médio").font(.caption).foregroundColor(.secondary); Text(String(format: "%.1f", b)).font(.headline) }
            }
        }
        .padding(8)
        .background(Color(.secondarySystemBackground))
        .cornerRadius(8)
    }
}

struct ResultCard: View {
    let row: ValidationRow
    var bleuWeight: Double = 0.3
    var bleuThreshold: Double = 70.0
    @State private var expanded = false
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            let quality: Double = {
                let b = row.bleu ?? 0.0
                return max(0.0, min(100.0, (1.0 - bleuWeight) * row.score + bleuWeight * b))
            }()
            let passBLEU = (row.bleu ?? 0.0) >= bleuThreshold
            HStack {
                Text(row.provider).font(.headline)
                Spacer()
                Text(String(format: "Adherence: %.1f%%", row.score)).font(.caption)
                    .foregroundColor(row.score >= 80 ? .green : (row.score >= 60 ? .orange : .red))
            }
            HStack(spacing: 12) {
                Label("\(row.latencyMs) ms", systemImage: "clock")
                Label("in \(row.tokensIn)", systemImage: "text.alignleft")
                Label("out \(row.tokensOut)", systemImage: "text.justify")
                Text(String(format: "$ %.4f", row.costUSD)).bold()
            }.font(.caption)
            if let b = row.bleu {
                HStack(spacing: 12) {
                    Text(String(format: "BLEU: %.1f", b)).font(.caption).foregroundColor(.blue)
                    Text(passBLEU ? "Conforme ref ✅" : "Abaixo do limiar ⚠️").font(.caption)
                        .foregroundColor(passBLEU ? .green : .orange)
                    Text(String(format: "Qualidade: %.1f%%", quality)).font(.caption)
                        .foregroundColor(quality >= 80 ? .green : (quality >= 60 ? .orange : .red))
                }
            }
            Button(expanded ? "Esconder resposta" : "Ver resposta") { expanded.toggle() }.buttonStyle(.bordered)
            if expanded { Text(row.response).font(.system(.footnote, design: .monospaced)).padding(8).background(Color(.tertiarySystemBackground)).cornerRadius(6) }
        }
        .padding(12)
        .background(Color(.secondarySystemBackground))
        .cornerRadius(10)
    }
}
