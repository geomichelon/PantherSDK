import 'dart:convert';
import 'dart:io';
import 'package:flutter/material.dart';
import 'ffi.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:url_launcher/url_launcher.dart';

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
  String? proofHash;
feature/ProofSeal
  String? explorerUrl;
  String? contractUrl;
  String? anchorStatus;
  String? explorerUrl;

  String? anchorStatus;
 main

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
            proofHash = proof['combined_hash'] as String;
            validationLines.add('Proof: $proofHash');
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
              const Text('Backend API'),
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
feature/ProofSeal
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

main
              ],
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
feature/ProofSeal
      final anchored = (obj['anchored'] as bool?) ?? false;
      final cu = obj['contract_url'] as String?;
      setState(() {
        anchorStatus = 'Anchored: ${anchored ? 'true' : 'false'}';
        contractUrl = cu;
      });

      final anchored = obj['anchored'] as bool? ?? false;
      setState(() { anchorStatus = 'Anchored: $anchored'; });
main
    } catch (e) {
      setState(() { anchorStatus = 'Status error: $e'; });
    } finally {
      client.close(force: true);
    }
  }
}
