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
  String validation = '';
  String prompt = 'Explain insulin function';

  @override
  void initState() {
    super.initState();
    panther.init();
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('PantherSDK Flutter Sample')),
        body: Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
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
              Padding(
                padding: const EdgeInsets.all(8.0),
                child: TextField(
                  decoration: const InputDecoration(labelText: 'Prompt'),
                  onChanged: (v) => prompt = v,
                ),
              ),
              ElevatedButton(
                onPressed: () {
                  setState(() { validation = panther.validate(prompt); });
                },
                child: const Text('Validate'),
              ),
              const SizedBox(height: 8),
              Padding(
                padding: const EdgeInsets.all(8.0),
                child: TextField(
                  decoration: const InputDecoration(labelText: 'Reference'),
                  onChanged: (v) => reference = v,
                ),
              ),
              ElevatedButton(
                onPressed: () {
                  setState(() { bleu = panther.metricsBleu(reference, output); });
                },
                child: const Text('BLEU'),
              ),
              const SizedBox(height: 8),
              Text('Output: ' + output),
              Text('BLEU: ' + bleu.toStringAsFixed(3)),
              const SizedBox(height: 8),
              const Text('Validation (JSON):'),
              Text(validation),
            ],
          ),
        ),
      ),
    );
  }
}
