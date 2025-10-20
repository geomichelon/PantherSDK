import 'dart:convert';
import 'dart:io';
import 'package:flutter/material.dart';
import 'ffi.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() => runApp(const MyApp());

const defaultCostRulesJson = '[\n'
    '  {"match": "openai:gpt-4o-mini",  "usd_per_1k_in": 0.15, "usd_per_1k_out": 0.60},\n'
    '  {"match": "openai:gpt-4.1-mini", "usd_per_1k_in": 0.30, "usd_per_1k_out": 1.20},\n'
    '  {"match": "openai:gpt-4.1",      "usd_per_1k_in": 5.00,  "usd_per_1k_out": 15.00},\n'
    '  {"match": "openai:gpt-4o",       "usd_per_1k_in": 5.00,  "usd_per_1k_out": 15.00},\n'
    '  {"match": "openai:chatgpt-5",    "usd_per_1k_in": 5.00,  "usd_per_1k_out": 15.00},\n'
    '  {"match": "ollama:llama3",       "usd_per_1k_in": 0.00,  "usd_per_1k_out": 0.00},\n'
    '  {"match": "ollama:phi3",         "usd_per_1k_in": 0.00,  "usd_per_1k_out": 0.00},\n'
    '  {"match": "ollama:mistral",      "usd_per_1k_in": 0.00,  "usd_per_1k_out": 0.00}\n'
    ']';
const openAIModels = [
  'gpt-4o-mini',
  'gpt-4.1-mini',
  'gpt-4.1',
  'gpt-4o',
  'chatgpt-5',
];
const ollamaModels = [
  'llama3',
  'phi3',
  'mistral',
];

class MyApp extends StatefulWidget {
  const MyApp({super.key});
  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  final panther = PantherFFI();
  List<String> validationLines = [];
  String? proofHash;
  String? explorerUrl;
  String? contractUrl;
  String? anchorStatus;

  final List<Map<String, dynamic>> providerPresets = [
    {'label': 'OpenAI', 'type': 'openai', 'base': 'https://api.openai.com', 'model': 'gpt-4o-mini', 'requiresKey': true},
    {'label': 'Groq', 'type': 'openai', 'base': 'https://api.groq.com/openai/v1', 'model': 'llama3-70b-8192', 'requiresKey': true},
    {'label': 'Together', 'type': 'openai', 'base': 'https://api.together.xyz/v1', 'model': 'meta-llama/Meta-Llama-3.1-70B-Instruct', 'requiresKey': true},
    {'label': 'Mistral', 'type': 'openai', 'base': 'https://api.mistral.ai', 'model': 'mistral-small-latest', 'requiresKey': true},
    {'label': 'Ollama', 'type': 'ollama', 'base': 'http://127.0.0.1:11434', 'model': 'llama3', 'requiresKey': false},
  ];

  String selectedPreset = 'OpenAI';
  bool showApiKey = true;
  bool includeLocalOllama = false;

  late TextEditingController promptController;
  late TextEditingController baseController;
  late TextEditingController modelController;
  late TextEditingController apiKeyController;
  late TextEditingController localBaseController;
  late TextEditingController localModelController;
  late TextEditingController apiBaseController;
  late TextEditingController apiKeyControllerApi;
  late TextEditingController plagCorpusController;
  late TextEditingController plagCandidateController;
  late TextEditingController plagNgramController;
  late TextEditingController costRulesController;
  late TextEditingController openAIBaseController;
  late TextEditingController openAIModelController;
  late TextEditingController openAIKeyController;
  late TextEditingController ollamaBaseController;
  late TextEditingController ollamaModelController;
  late TextEditingController guidelinesController;
  late TextEditingController guidelinesUrlController;
  List<String> simLines = [];
  double? plagScore;
  double? trustIndex;
  double? biasScore;
  List<dynamic> lastResults = [];
  String mode = 'proof'; // single | multi | proof
  String provider = 'openai'; // openai | ollama | anthropic | default

  @override
  void initState() {
    super.initState();
    panther.init();
    promptController = TextEditingController(text: 'Explain insulin function');
    baseController = TextEditingController(text: providerPresets.first['base'] as String);
    modelController = TextEditingController(text: providerPresets.first['model'] as String);
    apiKeyController = TextEditingController();
    localBaseController = TextEditingController(text: 'http://127.0.0.1:11434');
    localModelController = TextEditingController(text: 'llama3');
    apiBaseController = TextEditingController(text: (Platform.isAndroid ? 'http://10.0.2.2:8000' : 'http://127.0.0.1:8000'));
    apiKeyControllerApi = TextEditingController();
    plagCorpusController = TextEditingController(text: 'Insulin regulates glucose in the blood.\nVitamin C supports the immune system.');
    plagCandidateController = TextEditingController(text: 'Insulin regulates glucose in the blood.');
    plagNgramController = TextEditingController(text: '3');
    costRulesController = TextEditingController(text: defaultCostRulesJson);
    openAIBaseController = TextEditingController(text: 'https://api.openai.com');
    openAIModelController = TextEditingController(text: 'gpt-4o-mini');
    openAIKeyController = TextEditingController();
    ollamaBaseController = TextEditingController(text: 'http://127.0.0.1:11434');
    ollamaModelController = TextEditingController(text: 'llama3');
    guidelinesController = TextEditingController();
    guidelinesUrlController = TextEditingController();
    indexNameController = TextEditingController(text: 'default');
    SharedPreferences.getInstance().then((prefs) {
      final s = prefs.getString('panther.cost_rules');
      if (s != null && s.trim().isNotEmpty) {
        setState(() { costRulesController.text = s; });
      }
      // Load provider session
      final p = prefs.getString('prov.type');
      if (p != null && p.trim().isNotEmpty) {
        setState(() { selectedPreset = p; });
      }
      final oBase = prefs.getString('prov.openai.base'); if (oBase != null) setState(() { openAIBaseController.text = oBase; });
      final oModel = prefs.getString('prov.openai.model'); if (oModel != null) setState(() { openAIModelController.text = oModel; });
      final oKey = prefs.getString('prov.openai.key'); if (oKey != null) setState(() { openAIKeyController.text = oKey; });
      final olBase = prefs.getString('prov.ollama.base'); if (olBase != null) setState(() { ollamaBaseController.text = olBase; });
      final olModel = prefs.getString('prov.ollama.model'); if (olModel != null) setState(() { ollamaModelController.text = olModel; });
    });
  }

  @override
  void dispose() {
    promptController.dispose();
    baseController.dispose();
    modelController.dispose();
    apiKeyController.dispose();
    localBaseController.dispose();
    localModelController.dispose();
    apiBaseController.dispose();
    apiKeyControllerApi.dispose();
    plagCorpusController.dispose();
    plagCandidateController.dispose();
    plagNgramController.dispose();
    costRulesController.dispose();
    openAIBaseController.dispose();
    openAIModelController.dispose();
    openAIKeyController.dispose();
    ollamaBaseController.dispose();
    ollamaModelController.dispose();
    guidelinesController.dispose();
    guidelinesUrlController.dispose();
    indexNameController.dispose();
    super.dispose();
  }

  void _applyPreset(String label) {
    final preset = providerPresets.firstWhere((p) => p['label'] == label);
    setState(() {
      selectedPreset = label;
      baseController.text = preset['base'] as String;
      modelController.text = preset['model'] as String;
      showApiKey = preset['requiresKey'] as bool;
      if (!showApiKey) apiKeyController.text = '';
    });
  }

  void _runValidation() {
    final preset = providerPresets.firstWhere((p) => p['label'] == selectedPreset);
    final base = baseController.text.trim();
    final model = modelController.text.trim();
    final prompt = promptController.text.trim();

    if (prompt.isEmpty) {
      setState(() { validationLines = ['Enter a prompt.']; });
      return;
    }
    if (showApiKey && apiKeyController.text.trim().isEmpty) {
      setState(() { validationLines = ['API key required for $selectedPreset']; });
      return;
    }

    final providers = <Map<String, dynamic>>[
      {
        'type': preset['type'],
        'base_url': base,
        'model': model,
        if (showApiKey && apiKeyController.text.trim().isNotEmpty)
          'api_key': apiKeyController.text.trim(),
      }
    ];
    if (includeLocalOllama) {
      providers.add({
        'type': 'ollama',
        'base_url': localBaseController.text.trim(),
        'model': localModelController.text.trim(),
      });
    }

    String raw;
    if (mode == 'single') {
      if (provider == 'openai') {
        raw = panther.validateOpenAI(prompt, openAIKeyController.text.trim(), openAIModelController.text.trim(), openAIBaseController.text.trim());
      } else if (provider == 'ollama') {
        raw = panther.validateOllama(prompt, ollamaBaseController.text.trim(), ollamaModelController.text.trim());
      } else {
        raw = panther.validate(prompt);
      }
    } else {
      final pjson = jsonEncode(providers);
      if (mode == 'multi') {
        raw = (guidelinesController.text.trim().isNotEmpty)
            ? panther.validateCustom(prompt, pjson, guidelinesController.text.trim())
            : panther.validateMulti(prompt, pjson);
      } else {
        raw = (guidelinesController.text.trim().isNotEmpty)
            ? panther.validateCustomWithProof(prompt, pjson, guidelinesController.text.trim())
            : panther.validateMultiWithProof(prompt, pjson);
      }
    }
    try {
      final decoded = jsonDecode(raw);
      if (decoded is Map && decoded.containsKey('results')) {
        final results = decoded['results'] as List<dynamic>;
        final tin = panther.tokenCount(prompt);
        setState(() {
          validationLines = results.map((entry) {
            final name = entry['provider_name'] ?? '?';
            final score = (entry['adherence_score'] ?? 0).toDouble();
            final latency = entry['latency_ms'] ?? 0;
            final text = entry['raw_text'] is String ? (entry['raw_text'] as String) : '';
            final tout = panther.tokenCount(text);
            final rules = (costRulesController.text.trim().isEmpty) ? defaultCostRulesJson : costRulesController.text;
            final cost = panther.calculateCost(tin, tout, '$name', rules);
            return '$name – ${score.toStringAsFixed(1)}% – $latency ms – $tin/$tout tok – \$${cost.toStringAsFixed(4)}';
          }).cast<String>().toList();
          final proof = decoded['proof'] as Map<String, dynamic>?;
          if (proof != null && proof['combined_hash'] is String) {
            proofHash = proof['combined_hash'] as String;
            validationLines.add('Proof: $proofHash');
          }
          lastResults = results;
        });
        return;
      }
    } catch (_) {}
    setState(() { validationLines = [raw]; });
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('PantherSDK Flutter Sample')),
        body: SingleChildScrollView(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('PantherSDK Flutter Sample', style: const TextStyle(fontWeight: FontWeight.bold)),
              const SizedBox(height: 16),
              const Text('Compliance (Bias)'),
              const SizedBox(height: 4),
              Row(children: [
                ElevatedButton(
                  onPressed: () async {
                    try {
                      // Use validation lines as samples for simplicity
                      final samples = validationLines;
                      final res = panther.biasDetect(samples);
                      final obj = jsonDecode(res) as Map<String, dynamic>;
                      final b = (obj['bias_score'] as num?)?.toDouble();
                      double avg = 0.0;
                      if (lastResults.isNotEmpty) {
                        final vals = lastResults.map((e) => ((e['adherence_score'] ?? 0) as num).toDouble()/100.0).toList();
                        avg = vals.isNotEmpty ? (vals.reduce((a,b)=>a+b) / vals.length) : 0.0;
                      }
                      setState(() {
                        biasScore = b;
                        trustIndex = (b == null) ? avg : (avg * (1.0 - b)).clamp(0.0, 1.0);
                      });
                    } catch (_) {}
                  },
                  child: const Text('Analyze Bias'),
                ),
                const SizedBox(width: 12),
                if (biasScore != null) Text('bias_score: ${biasScore!.toStringAsFixed(3)}'),
                const SizedBox(width: 12),
                if (trustIndex != null) Text('trust_index: ${(trustIndex!*100).toStringAsFixed(1)}%'),
              ]),
              TextField(
                controller: promptController,
                decoration: const InputDecoration(labelText: 'Prompt'),
              ),
              const SizedBox(height: 12),
              const Text('Execution'),
              Row(children: [
                ChoiceChip(label: const Text('Single'), selected: mode=='single', onSelected: (_){ setState(()=> mode='single'); }),
                const SizedBox(width: 8),
                ChoiceChip(label: const Text('Multi'), selected: mode=='multi', onSelected: (_){ setState(()=> mode='multi'); }),
                const SizedBox(width: 8),
                ChoiceChip(label: const Text('With Proof'), selected: mode=='proof', onSelected: (_){ setState(()=> mode='proof'); }),
              ]),
              const SizedBox(height: 8),
              Row(children: [
                ChoiceChip(label: const Text('OpenAI'), selected: provider=='openai', onSelected: (_){ setState(()=> provider='openai'); }),
                const SizedBox(width: 8),
                ChoiceChip(label: const Text('Ollama'), selected: provider=='ollama', onSelected: (_){ setState(()=> provider='ollama'); }),
                const SizedBox(width: 8),
                ChoiceChip(label: const Text('Anthropic'), selected: provider=='anthropic', onSelected: (_){ setState(()=> provider='anthropic'); }),
                const SizedBox(width: 8),
                ChoiceChip(label: const Text('Default'), selected: provider=='default', onSelected: (_){ setState(()=> provider='default'); }),
              ]),
              if (provider=='openai') ...[
                TextField(controller: openAIKeyController, decoration: const InputDecoration(labelText: 'OpenAI API Key'), obscureText: true),
                const SizedBox(height: 8),
                TextField(controller: openAIBaseController, decoration: const InputDecoration(labelText: 'Base URL')),
                const SizedBox(height: 8),
                TextField(controller: openAIModelController, decoration: const InputDecoration(labelText: 'Model (e.g., gpt-4o-mini)')),
                const SizedBox(height: 6),
                Wrap(
                  spacing: 8,
                  children: openAIModels.map((m) => ChoiceChip(
                    label: Text(m),
                    selected: openAIModelController.text.trim()==m,
                    onSelected: (_){ setState(()=> openAIModelController.text=m); },
                  )).toList(),
                ),
              ]
              else if (provider=='ollama') ...[
                TextField(controller: ollamaBaseController, decoration: const InputDecoration(labelText: 'Ollama Base (http://127.0.0.1:11434)')),
                const SizedBox(height: 8),
                TextField(controller: ollamaModelController, decoration: const InputDecoration(labelText: 'Ollama Model (e.g., llama3)')),
                const SizedBox(height: 6),
                Wrap(
                  spacing: 8,
                  children: ollamaModels.map((m) => ChoiceChip(
                    label: Text(m),
                    selected: ollamaModelController.text.trim()==m,
                    onSelected: (_){ setState(()=> ollamaModelController.text=m); },
                  )).toList(),
                ),
              ]
              else if (provider=='anthropic') ...[
                TextField(controller: anthropicKeyController, decoration: const InputDecoration(labelText: 'Anthropic API Key'), obscureText: true),
                const SizedBox(height: 8),
                TextField(controller: anthropicBaseController, decoration: const InputDecoration(labelText: 'Base URL (https://api.anthropic.com)')),
                const SizedBox(height: 8),
                TextField(controller: anthropicModelController, decoration: const InputDecoration(labelText: 'Model (e.g., claude-3-5-sonnet-latest)')),
                const SizedBox(height: 6),
                Wrap(
                  spacing: 8,
                  children: ['claude-3-5-sonnet-latest','claude-3-opus-latest','claude-3-haiku-latest'].map((m) => ChoiceChip(
                    label: Text(m),
                    selected: anthropicModelController.text.trim()==m,
                    onSelected: (_){ setState(()=> anthropicModelController.text=m); },
                  )).toList(),
                ),
              ]
              else ...[
                const Text('Using environment providers (Default)', style: TextStyle(color: Colors.grey)),
              ],
              const SizedBox(height: 8),
              const Text('Guidelines'),
              TextField(
                controller: guidelinesController,
                minLines: 3,
                maxLines: 8,
                decoration: const InputDecoration(hintText: '[ { "topic": ..., "expected_terms": [...] } ]', border: OutlineInputBorder()),
              ),
              const SizedBox(height: 6),
              Row(children: [
                Expanded(child: TextField(controller: guidelinesUrlController, decoration: const InputDecoration(hintText: 'https://…/guidelines.json', border: OutlineInputBorder()))),
                const SizedBox(width: 8),
                ElevatedButton(onPressed: () async {
                  try {
                    final client = HttpClient();
                    final req = await client.getUrl(Uri.parse(guidelinesUrlController.text.trim()));
                    final resp = await req.close();
                    final body = await resp.transform(const Utf8Decoder()).join();
                    setState(() { guidelinesController.text = body; });
                  } catch (_) {}
                }, child: const Text('Load')),
                const SizedBox(width: 8),
                ElevatedButton(onPressed: () {
                  final json = guidelinesController.text.trim(); final q = promptController.text.trim();
                  if (json.isEmpty || q.isEmpty) { setState(() { simLines = ['Provide guidelines JSON and prompt']; }); return; }
                  final n = panther.guidelinesIngest(json);
                  if (n <= 0) { setState(() { simLines = ['No items ingested']; }); return; }
          final out = panther.guidelinesScores(q, 5, 'hybrid');
          try {
            final arr = jsonDecode(out) as List<dynamic>;
            setState(() {
              simLines = arr.map((e) {
                final t = e['topic']?.toString() ?? '?';
                final s = (e['score'] ?? 0).toDouble();
                final bow = (e['bow'] ?? 0).toDouble();
                final jac = (e['jaccard'] ?? 0).toDouble();
                return '$t – ${s.toStringAsFixed(3)} (bow ${bow.toStringAsFixed(3)}, jac ${jac.toStringAsFixed(3)})';
              }).cast<String>().toList();
            });
          } catch (_) { setState(() { simLines = [out]; }); }
        }, child: const Text('Fetch + scores'))
              ]),
              if (simLines.isNotEmpty) ...[
                const SizedBox(height: 6),
                Column(crossAxisAlignment: CrossAxisAlignment.start, children: simLines.map((l) => Text(l)).toList()),
              ],
              const SizedBox(height: 6),
              Row(children:[
                Expanded(child: TextField(controller: indexNameController, decoration: const InputDecoration(hintText: 'index name', border: OutlineInputBorder()))),
                const SizedBox(width: 8),
                ElevatedButton(onPressed: () {
                  final name = (indexNameController.text.trim().isEmpty ? 'default' : indexNameController.text.trim());
                  final json = guidelinesController.text.trim();
                  if (json.isEmpty) { setState(() { simLines = ['Provide guidelines JSON']; }); return; }
                  final rc = panther.guidelinesSave(name, json);
                  setState(() { simLines = [rc == 0 ? 'Index saved: '+name : 'Save failed']; });
                }, child: const Text('Save Index')),
                const SizedBox(width: 8),
                ElevatedButton(onPressed: () {
                  final name = (indexNameController.text.trim().isEmpty ? 'default' : indexNameController.text.trim());
                  final n = panther.guidelinesLoad(name);
                  setState(() { simLines = [n > 0 ? 'Index loaded: '+name+' ('+n.toString()+')' : 'Load failed or empty']; });
                }, child: const Text('Load Index')),
              ]),
              const SizedBox(height: 12),
              DropdownButton<String>(
                value: selectedPreset,
                isExpanded: true,
                items: providerPresets
                    .map((p) => DropdownMenuItem<String>(
                          value: p['label'] as String,
                          child: Text(p['label'] as String),
                        ))
                    .toList(),
                onChanged: (value) {
                  if (value != null) _applyPreset(value);
                },
              ),
              const SizedBox(height: 8),
              TextField(
                controller: baseController,
                decoration: const InputDecoration(labelText: 'Base URL'),
              ),
              const SizedBox(height: 8),
              TextField(
                controller: modelController,
                decoration: const InputDecoration(labelText: 'Model'),
              ),
              if (showApiKey) ...[
                const SizedBox(height: 8),
                TextField(
                  controller: apiKeyController,
                  decoration: const InputDecoration(labelText: 'API Key'),
                  obscureText: true,
                ),
              ],
              const SizedBox(height: 8),
              Row(children:[
                ElevatedButton(onPressed: () async {
                  final prefs = await SharedPreferences.getInstance();
                  await prefs.setString('prov.type', selectedPreset);
                  await prefs.setString('prov.openai.base', openAIBaseController.text.trim());
                  await prefs.setString('prov.openai.model', openAIModelController.text.trim());
                  await prefs.setString('prov.openai.key', openAIKeyController.text.trim());
                  await prefs.setString('prov.ollama.base', ollamaBaseController.text.trim());
                  await prefs.setString('prov.ollama.model', ollamaModelController.text.trim());
                }, child: const Text('Save Provider Session'))
              ]),
              const SizedBox(height: 8),
              SwitchListTile(
                title: const Text('Include local Ollama'),
                value: includeLocalOllama,
                onChanged: (val) => setState(() => includeLocalOllama = val),
              ),
              if (includeLocalOllama) ...[
                TextField(
                  controller: localBaseController,
                  decoration: const InputDecoration(labelText: 'Ollama Base URL'),
                ),
                const SizedBox(height: 8),
                TextField(
                  controller: localModelController,
                  decoration: const InputDecoration(labelText: 'Ollama Model'),
                ),
              ],
              const SizedBox(height: 12),
              const Text('Backend API'),
              const SizedBox(height: 8),
              const Text('Cost Rules (JSON)'),
              Row(children: [
                ElevatedButton(
                  onPressed: () => setState(() { costRulesController.text = defaultCostRulesJson; }),
                  child: const Text('Restore Default'),
                ),
                const SizedBox(width: 8),
                ElevatedButton(
                  onPressed: () async {
                    final prefs = await SharedPreferences.getInstance();
                    await prefs.setString('panther.cost_rules', costRulesController.text);
                  },
                  child: const Text('Save'),
                ),
              ]),
              const SizedBox(height: 6),
              TextField(
                controller: costRulesController,
                minLines: 4,
                maxLines: 12,
                decoration: const InputDecoration(border: OutlineInputBorder()),
              ),
              const SizedBox(height: 8),
              const Text('Cost Rules (JSON)'),
              Row(children: [
                ElevatedButton(
                  onPressed: () => setState(() { costRulesController.text = defaultCostRulesJson; }),
                  child: const Text('Restore Default'),
                ),
              ]),
              const SizedBox(height: 6),
              TextField(
                controller: costRulesController,
                minLines: 4,
                maxLines: 12,
                decoration: const InputDecoration(border: OutlineInputBorder()),
              ),
              TextField(
                controller: apiBaseController,
                decoration: const InputDecoration(labelText: 'API Base (e.g., http://127.0.0.1:8000)'),
              ),
              const SizedBox(height: 8),
              TextField(
                controller: apiKeyControllerApi,
                decoration: const InputDecoration(labelText: 'API Key (X-API-Key, optional)'),
                obscureText: true,
              ),
              const SizedBox(height: 12),
              ElevatedButton(
                onPressed: _runValidation,
                child: const Text('Validate'),
              ),
              if (proofHash != null) ...[
                const SizedBox(height: 8),
                ElevatedButton(
                  onPressed: _anchorProof,
                  child: const Text('Anchor Proof (API)'),
                ),
                const SizedBox(height: 8),
                ElevatedButton(
                  onPressed: _checkStatus,
                  child: const Text('Check Status (API)'),
                ),
                if (explorerUrl != null) ...[
                  const SizedBox(height: 8),
                  ElevatedButton(
                    onPressed: () async {
                      final url = Uri.parse(explorerUrl!);
                      if (await canLaunchUrl(url)) {
                        await launchUrl(url, mode: LaunchMode.externalApplication);
                      }
                    },
                    child: const Text('View on Explorer'),
                  ),
                ],
                if (contractUrl != null) ...[
                  const SizedBox(height: 8),
                  ElevatedButton(
                    onPressed: () async {
                      final url = Uri.parse(contractUrl!);
                      if (await canLaunchUrl(url)) {
                        await launchUrl(url, mode: LaunchMode.externalApplication);
                      }
                    },
                    child: const Text('View Contract'),
                  ),
                ],
              ],

              const SizedBox(height: 16),
              const Text('Plagiarism (Jaccard n-gram)'),
              const SizedBox(height: 4),
              const Text('Corpus (one per line):'),
              TextField(
                controller: plagCorpusController,
                minLines: 3,
                maxLines: 6,
                decoration: const InputDecoration(border: OutlineInputBorder()),
              ),
              const SizedBox(height: 8),
              const Text('Candidate text:'),
              TextField(
                controller: plagCandidateController,
                decoration: const InputDecoration(border: OutlineInputBorder()),
              ),
              const SizedBox(height: 8),
              Row(
                children: [
                  ElevatedButton(
                    onPressed: () {
                      final corpus = plagCorpusController.text.split('\n').map((e) => e.trim()).where((e) => e.isNotEmpty).toList();
                      final cand = plagCandidateController.text;
                      final n = int.tryParse(plagNgramController.text.trim()) ?? 3;
                      setState(() {
                        plagScore = panther.metricsPlagiarismNgram(corpus, cand, n);
                      });
                    },
                    child: const Text('Check Plagiarism'),
                  ),
                  const SizedBox(width: 12),
                  const Text('n-gram:'),
                  const SizedBox(width: 6),
                  SizedBox(
                    width: 60,
                    child: TextField(
                      controller: plagNgramController,
                      keyboardType: TextInputType.number,
                      decoration: const InputDecoration(border: OutlineInputBorder()),
                    ),
                  ),
                  const SizedBox(width: 12),
                  if (plagScore != null) Text('Score: ${plagScore!.toStringAsFixed(2)}'),
                ],
              ),
              const SizedBox(height: 16),
              const SizedBox(height: 16),
              const Text('Compliance (Bias):'),
              TextField(
                controller: TextEditingController(text: ''),
                onChanged: (_) {},
                decoration: const InputDecoration(hintText: 'Enter responses (one per line)', border: OutlineInputBorder()),
                minLines: 3,
                maxLines: 5,
                onSubmitted: (_) {},
              ),
              const SizedBox(height: 8),
              ElevatedButton(
                onPressed: () {
                  // For simplicity, reuse validation lines as samples
                  final samples = validationLines;
                  final res = panther.biasDetect(samples);
                  setState(() { validationLines.add('Bias: $res'); });
                },
                child: const Text('Analyze Bias'),
              ),
              const SizedBox(height: 16),
              const Text('Validation:'),
              ...validationLines.map((line) => Text(line)).toList(),
              if (anchorStatus != null) Text(anchorStatus!),
            ],
          ),
        ),
      ),
    );
  }

  void _anchorProof() async {
    final hash = proofHash;
    if (hash == null) return;
    final base = apiBaseController.text.trim().isEmpty
        ? (Platform.isAndroid ? 'http://10.0.2.2:8000' : 'http://127.0.0.1:8000')
        : apiBaseController.text.trim();
    final client = HttpClient();
    try {
      final req = await client.postUrl(Uri.parse('$base/proof/anchor'));
      req.headers.set(HttpHeaders.contentTypeHeader, 'application/json');
      final k = apiKeyControllerApi.text.trim();
      if (k.isNotEmpty) req.headers.set('X-API-Key', k);
      req.add(utf8.encode(jsonEncode({'hash': '0x$hash'})));
      final resp = await req.close();
      final body = await resp.transform(utf8.decoder).join();
      final obj = jsonDecode(body) as Map<String, dynamic>;
      final tx = obj['tx_hash'] as String?;
      final ex = obj['explorer_url'] as String?;
      setState(() {
        validationLines.add(tx != null ? 'Anchored tx: $tx' : 'Anchor failed');
        explorerUrl = ex;
      });
    } catch (e) {
      setState(() { validationLines.add('Anchor error: $e'); });
    } finally {
      client.close(force: true);
    }
  }

  void _checkStatus() async {
    final hash = proofHash;
    if (hash == null) return;
    final base = apiBaseController.text.trim().isEmpty
        ? (Platform.isAndroid ? 'http://10.0.2.2:8000' : 'http://127.0.0.1:8000')
        : apiBaseController.text.trim();
    final client = HttpClient();
    try {
      final req = await client.getUrl(Uri.parse('$base/proof/status?hash=0x$hash'));
      final k = apiKeyControllerApi.text.trim();
      if (k.isNotEmpty) req.headers.set('X-API-Key', k);
      final resp = await req.close();
      final body = await resp.transform(utf8.decoder).join();
      final obj = jsonDecode(body) as Map<String, dynamic>;
      final anchored = (obj['anchored'] as bool?) ?? false;
      final cu = obj['contract_url'] as String?;
      setState(() {
        anchorStatus = 'Anchored: ${anchored ? 'true' : 'false'}';
        contractUrl = cu;
      });
    } catch (e) {
      setState(() { anchorStatus = 'Status error: $e'; });
    } finally {
      client.close(force: true);
    }
  }
}
