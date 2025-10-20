import SwiftUI
import UIKit

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
    var totalCost: Double { rows.reduce(0) { $0 + $1.costUSD } }
    var avgLatency: Int { guard !rows.isEmpty else { return 0 }; return rows.map { $0.latencyMs }.reduce(0, +) / rows.count }
    var body: some View {
        HStack {
            VStack(alignment: .leading) { Text("Melhor provedor").font(.caption).foregroundColor(.secondary); Text(best?.provider ?? "–").font(.headline) }
            Spacer()
            VStack(alignment: .trailing) { Text("Latência média").font(.caption).foregroundColor(.secondary); Text("\(avgLatency) ms").font(.headline) }
            Spacer()
            VStack(alignment: .trailing) { Text("Custo total").font(.caption).foregroundColor(.secondary); Text(String(format: "$ %.4f", totalCost)).font(.headline) }
        }
        .padding(8)
        .background(Color(UIColor.secondarySystemBackground))
        .cornerRadius(8)
    }
}

struct ResultCard: View {
    let row: ValidationRow
    @State private var expanded = false
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack { Text(row.provider).font(.headline); Spacer(); Text(String(format: "%.1f%%", row.score)).font(.subheadline).foregroundColor(row.score >= 80 ? .green : (row.score >= 60 ? .orange : .red)) }
            HStack(spacing: 12) { Label("\(row.latencyMs) ms", systemImage: "clock"); Label("in \(row.tokensIn)", systemImage: "text.alignleft"); Label("out \(row.tokensOut)", systemImage: "text.justify"); Text(String(format: "$ %.4f", row.costUSD)).bold() }.font(.caption)
            Button(expanded ? "Esconder resposta" : "Ver resposta") { expanded.toggle() }.buttonStyle(.bordered)
            if expanded { Text(row.response).font(.system(.footnote, design: .monospaced)).padding(8).background(Color(UIColor.tertiarySystemBackground)).cornerRadius(6) }
        }
        .padding(12)
        .background(Color(UIColor.secondarySystemBackground))
        .cornerRadius(10)
    }
}
