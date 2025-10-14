import 'dart:convert';
import 'package:flutter/material.dart';
import 'ffi.dart';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatefulWidget {
  const MyApp({super.key});
  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  final panther = PantherFFI();
  String output = '';
  String reference = '';
  double bleu = 0;
  List<String> validationLines = [];

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
  }

  @override
  void dispose() {
    promptController.dispose();
    baseController.dispose();
    modelController.dispose();
    apiKeyController.dispose();
    localBaseController.dispose();
    localModelController.dispose();
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

    final raw = panther.validateMultiWithProof(prompt, jsonEncode(providers));
    try {
      final decoded = jsonDecode(raw);
      if (decoded is Map && decoded.containsKey('results')) {
        final results = decoded['results'] as List<dynamic>;
        setState(() {
          validationLines = results.map((entry) {
            final name = entry['provider_name'] ?? '?';
            final score = (entry['adherence_score'] ?? 0).toDouble();
            final latency = entry['latency_ms'] ?? 0;
            return '$name – ${score.toStringAsFixed(1)}% – $latency ms';
          }).cast<String>().toList();
          final proof = decoded['proof'] as Map<String, dynamic>?;
          if (proof != null && proof['combined_hash'] is String) {
            validationLines.add('Proof: ${proof['combined_hash']}');
          }
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
              ElevatedButton(
                onPressed: () {
                  setState(() {
                    output = panther.generate('hello from flutter');
                  });
                },
                child: const Text('Generate'),
              ),
              const SizedBox(height: 16),
              TextField(
                controller: promptController,
                decoration: const InputDecoration(labelText: 'Prompt'),
              ),
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
              ElevatedButton(
                onPressed: _runValidation,
                child: const Text('Validate'),
              ),
              const SizedBox(height: 16),
              TextField(
                decoration: const InputDecoration(labelText: 'Reference (BLEU)'),
                onChanged: (v) => reference = v,
              ),
              const SizedBox(height: 8),
              ElevatedButton(
                onPressed: () {
                  setState(() {
                    bleu = panther.metricsBleu(reference, output);
                  });
                },
                child: const Text('BLEU'),
              ),
              const SizedBox(height: 16),
              Text('Output: $output'),
              Text('BLEU: ${bleu.toStringAsFixed(3)}'),
              const SizedBox(height: 16),
              const Text('Validation:'),
              ...validationLines.map((line) => Text(line)).toList(),
            ],
          ),
        ),
      ),
    );
  }
}
