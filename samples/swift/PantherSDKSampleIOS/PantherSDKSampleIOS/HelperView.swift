import SwiftUI

struct HelperView: View {
    var body: some View {
        NavigationView {
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    Group {
                        Text("Visão geral").font(.headline)
                        Text("Esta tela valida e compara respostas de LLMs para um prompt, calculando aderência às diretrizes (adherence), métricas de conteúdo (ex.: BLEU) e custos estimados. Você pode rodar um único provedor (Single), vários provedores (Multi) ou Multi com prova de integridade (With Proof).")
                    }

                    Group {
                        Text("Prompt").font(.headline)
                        Text("Digite o texto da sua consulta. Ele é enviado aos provedores selecionados. A contagem de tokens é aproximada (por palavras) e serve apenas para estimativa de custos.")
                    }

                    Group {
                        Text("Execução").font(.headline)
                        Text("Modo")
                            .font(.subheadline).foregroundColor(.secondary)
                        Text("• Single: escolhe 1 provedor/modelo e executa.")
                        Text("• Multi: executa vários provedores/modelos em paralelo e compara.")
                        Text("• With Proof: igual ao Multi, mas retorna também um objeto de prova (hashes) para auditoria.")

                        Text("Provider (apenas Single)")
                            .font(.subheadline).foregroundColor(.secondary)
                        Text("Selecione OpenAI/Ollama/Anthropic/Default. Em seguida, preencha chave/API base/modelo conforme necessário.")
                    }

                    Group {
                        Text("Custos (estimativa)").font(.headline)
                        Text("A estimativa usa uma tabela editável de preços por 1k tokens de entrada/saída. A contagem de tokens é heurística (espaços). Ajuste os valores na opção ‘Editar Tabela’ para refletir seus contratos reais.")
                    }

                    Group {
                        Text("Validar").font(.headline)
                        Text("O botão Validate dispara a execução no modo selecionado. Em Multi/With Proof, vários modelos são avaliados em paralelo.")
                    }

                    Group {
                        Text("Resultados").font(.headline)
                        Text("Cada cartão mostra: provedor, Adherence (0–100), BLEU (0–100), latência, tokens in/out e custo estimado. ")
                        Text("Qualidade (Adh + BLEU)")
                            .font(.subheadline).foregroundColor(.secondary)
                        Text("Você pode ponderar o BLEU na métrica de Qualidade via slider (Peso BLEU) e ajustar o limiar para o selo ‘Conforme ref’. Isso ajuda a avaliar proximidade com a redação neutra, além da cobertura de termos.")
                    }

                    Group {
                        Text("Neutral Reference (BLEU)").font(.headline)
                        Text("Insira um texto gabarito, técnico e imparcial, para comparar as respostas via BLEU. Você pode colar múltiplas referências (separadas por linha em branco); usamos o melhor BLEU.")
                        Text("Sem referência preenchida, o app calcula BLEU relativo usando a melhor resposta do próprio run como referência, útil para ranquear modelos mesmo sem um ‘padrão ouro’.")
                    }

                    Group {
                        Text("ROUGE‑L").font(.headline)
                        Text("Mede similaridade via maior subsequência comum (LCS) entre a resposta e uma referência. Valores típicos de 0 a 1 (quanto maior, mais parecido). É mais ‘recall‑orientado’ que o BLEU e tolera pequenas reordenações de palavras.")
                        Text("Uso no app: abra a tela Métricas (ícone de gráfico), cole a referência em ROUGE‑L e calcule para a resposta selecionada.")
                    }

                    Group {
                        Text("Fact coverage / Fact‑check").font(.headline)
                        Text("Coverage mede a fração de fatos declarados que aparecem (literalmente) na resposta; o Fact‑check avançado penaliza inconsistências.")
                        Text("Entradas: liste um fato por linha (ex.: ‘insulina reduz glicemia’). Na tela Métricas, cole essa lista em ‘Fatos’ e use os botões Coverage/Fact‑check.")
                        Text("Leitura: quanto maior, melhor. Use fatos curtos, objetivos e no mesmo idioma da resposta para reduzir falsos negativos.")
                    }

                    Group {
                        Text("Plágio (Jaccard)").font(.headline)
                        Text("Compara a resposta com um corpus (linhas) usando Jaccard de tokens. A variante n‑gram avalia sobreposição de sequências (n tokens).")
                        Text("Entradas: cole o corpus (uma amostra por linha) e ajuste o n‑gram (1–7). Resultados próximos de 1 indicam alta similaridade.")
                        Text("Uso: bom para detectar reaproveitamento literal do corpus. Para paráfrases, combine com ROUGE/ BLEU.")
                    }

                    Group {
                        Text("Compliance Report").font(.headline)
                        Text("Mostra um resumo dos modelos, a Adherence média e a análise de termos.")
                        Text("Análise de Termos")
                            .font(.subheadline).foregroundColor(.secondary)
                        Text("• Com diretrizes (URL): avaliamos a cobertura de cada tópico/termos do seu JSON.")
                        Text("• Sem diretrizes: derivamos termos da referência neutra (se preenchida) e mostramos cobertura por modelo + ranking dos melhores por termos.")
                    }

                    Group {
                        Text("Diretrizes por URL").font(.headline)
                        Text("Cole a URL (HTTPS) de um JSON de diretrizes. Formato esperado: uma lista de objetos com ‘topic’ (string) e ‘expected_terms’ (lista de strings). Ao carregar, a análise de termos usa exatamente seu JSON.")
                    }

                    Group {
                        Text("Prova (With Proof)").font(.headline)
                        Text("No modo With Proof, é retornado um objeto de prova (hashes) que permite verificar a integridade do relatório. Os botões Anchor/Status usam a API Python para ancorar o hash e consultar o estado.")
                    }

                    Group {
                        Text("Dicas rápidas").font(.headline)
                        Text("• Use Single para testar ajustes finos de um modelo; use Multi para comparativos.")
                        Text("• BLEU complementa a Adherence: Adherence verifica cobertura de termos; BLEU verifica proximidade com a redação neutra.")
                        Text("• Ajuste preços na tabela para estimativas de custo realistas.")
                    }
                }
                .padding()
            }
            .navigationTitle("Ajuda")
        }
    }
}

struct HelperView_Previews: PreviewProvider {
    static var previews: some View { HelperView() }
}
