import 'dart:ffi' as ffi;
import 'dart:io';
import 'package:ffi/ffi.dart' as pkg_ffi;

typedef _c_init = ffi.Int32 Function();
typedef _c_generate = ffi.Pointer<ffi.Char> Function(ffi.Pointer<ffi.Char>);
typedef _c_free = ffi.Void Function(ffi.Pointer<ffi.Char>);
typedef _c_metrics_bleu = ffi.Double Function(ffi.Pointer<ffi.Char>, ffi.Pointer<ffi.Char>);
typedef _c_metrics_plagiarism = ffi.Double Function(ffi.Pointer<ffi.Char>, ffi.Pointer<ffi.Char>);
typedef _c_metrics_plagiarism_ngram = ffi.Double Function(ffi.Pointer<ffi.Char>, ffi.Pointer<ffi.Char>, ffi.Int32);
typedef _c_validate = ffi.Pointer<ffi.Char> Function(ffi.Pointer<ffi.Char>);
typedef _c_validate_multi = ffi.Pointer<ffi.Char> Function(ffi.Pointer<ffi.Char>, ffi.Pointer<ffi.Char>);
typedef _c_version = ffi.Pointer<ffi.Char> Function();
typedef _c_validate_multi_with_proof = ffi.Pointer<ffi.Char> Function(ffi.Pointer<ffi.Char>, ffi.Pointer<ffi.Char>);
typedef _c_validate_custom_with_proof = ffi.Pointer<ffi.Char> Function(ffi.Pointer<ffi.Char>, ffi.Pointer<ffi.Char>, ffi.Pointer<ffi.Char>);
typedef _c_bias_detect = ffi.Pointer<ffi.Char> Function(ffi.Pointer<ffi.Char>);
typedef _c_token_count = ffi.Int32 Function(ffi.Pointer<ffi.Char>);
typedef _c_calculate_cost = ffi.Double Function(ffi.Int32, ffi.Int32, ffi.Pointer<ffi.Char>, ffi.Pointer<ffi.Char>);
typedef _c_validate_openai = ffi.Pointer<ffi.Char> Function(ffi.Pointer<ffi.Char>, ffi.Pointer<ffi.Char>, ffi.Pointer<ffi.Char>, ffi.Pointer<ffi.Char>);
typedef _c_validate_ollama = ffi.Pointer<ffi.Char> Function(ffi.Pointer<ffi.Char>, ffi.Pointer<ffi.Char>, ffi.Pointer<ffi.Char>);
typedef _c_validate_custom = ffi.Pointer<ffi.Char> Function(ffi.Pointer<ffi.Char>, ffi.Pointer<ffi.Char>, ffi.Pointer<ffi.Char>);
<<<<<<< HEAD
typedef _c_guidelines_ingest = ffi.Int32 Function(ffi.Pointer<ffi.Char>);
typedef _c_guidelines_similarity = ffi.Pointer<ffi.Char> Function(ffi.Pointer<ffi.Char>, ffi.Int32, ffi.Pointer<ffi.Char>);
typedef _c_guidelines_save = ffi.Int32 Function(ffi.Pointer<ffi.Char>, ffi.Pointer<ffi.Char>);
typedef _c_guidelines_load = ffi.Int32 Function(ffi.Pointer<ffi.Char>);
typedef _c_guidelines_embed_build = ffi.Int32 Function(ffi.Pointer<ffi.Char>);
=======
>>>>>>> origin/main

class PantherFFI {
  late final ffi.DynamicLibrary _lib;
  late final _c_init _init;
  late final _c_generate _generate;
  late final _c_free _free;
  late final _c_metrics_bleu _bleu;
  late final _c_metrics_plagiarism _plag;
  late final _c_metrics_plagiarism_ngram _plagNgram;
  late final _c_validate _validate;
  late final _c_validate_multi _validateMulti;
  late final _c_version _version;
  late final _c_validate_multi_with_proof _validateMultiWithProof;
  late final _c_validate_custom_with_proof _validateCustomWithProof;
  late final _c_bias_detect _biasDetect;
  late final _c_token_count _tokenCount;
  late final _c_calculate_cost _calculateCost;
  late final _c_validate_openai _validateOpenAI;
  late final _c_validate_ollama _validateOllama;
  late final _c_validate_custom _validateCustom;
<<<<<<< HEAD
  late final _c_guidelines_ingest _guidelinesIngest;
  late final _c_guidelines_similarity _guidelinesSimilarity;
  late final _c_guidelines_save _guidelinesSave;
  late final _c_guidelines_load _guidelinesLoad;
  late final _c_guidelines_embed_build _guidelinesBuildEmbeddings;
=======
>>>>>>> origin/main

  PantherFFI() {
    if (Platform.isAndroid) {
      _lib = ffi.DynamicLibrary.open('libpanther_ffi.so');
    } else if (Platform.isIOS) {
      // If linked statically, symbols may be in the process image
      _lib = ffi.DynamicLibrary.process();
    } else if (Platform.isMacOS) {
      _lib = ffi.DynamicLibrary.open('target/debug/libpanther_ffi.dylib');
    } else {
      throw UnsupportedError('Unsupported platform');
    }
    _init = _lib.lookupFunction<_c_init, _c_init>('panther_init');
    _generate = _lib.lookupFunction<_c_generate, _c_generate>('panther_generate');
    _free = _lib.lookupFunction<_c_free, _c_free>('panther_free_string');
    _bleu = _lib.lookupFunction<_c_metrics_bleu, _c_metrics_bleu>('panther_metrics_bleu');
    _plag = _lib.lookupFunction<_c_metrics_plagiarism, _c_metrics_plagiarism>('panther_metrics_plagiarism');
    _plagNgram = _lib.lookupFunction<_c_metrics_plagiarism_ngram, _c_metrics_plagiarism_ngram>('panther_metrics_plagiarism_ngram');
    _validate = _lib.lookupFunction<_c_validate, _c_validate>('panther_validation_run_default');
    _validateMulti = _lib.lookupFunction<_c_validate_multi, _c_validate_multi>('panther_validation_run_multi');
    _version = _lib.lookupFunction<_c_version, _c_version>('panther_version_string');
    _validateMultiWithProof = _lib.lookupFunction<_c_validate_multi_with_proof, _c_validate_multi_with_proof>('panther_validation_run_multi_with_proof');
    _validateCustomWithProof = _lib.lookupFunction<_c_validate_custom_with_proof, _c_validate_custom_with_proof>('panther_validation_run_custom_with_proof');
    _biasDetect = _lib.lookupFunction<_c_bias_detect, _c_bias_detect>('panther_bias_detect');
    _tokenCount = _lib.lookupFunction<_c_token_count, _c_token_count>('panther_token_count');
    _calculateCost = _lib.lookupFunction<_c_calculate_cost, _c_calculate_cost>('panther_calculate_cost');
    _validateOpenAI = _lib.lookupFunction<_c_validate_openai, _c_validate_openai>('panther_validation_run_openai');
    _validateOllama = _lib.lookupFunction<_c_validate_ollama, _c_validate_ollama>('panther_validation_run_ollama');
    _validateCustom = _lib.lookupFunction<_c_validate_custom, _c_validate_custom>('panther_validation_run_custom');
<<<<<<< HEAD
    _guidelinesIngest = _lib.lookupFunction<_c_guidelines_ingest, _c_guidelines_ingest>('panther_guidelines_ingest_json');
    _guidelinesSimilarity = _lib.lookupFunction<_c_guidelines_similarity, _c_guidelines_similarity>('panther_guidelines_similarity');
    _guidelinesSave = _lib.lookupFunction<_c_guidelines_save, _c_guidelines_save>('panther_guidelines_save_json');
    _guidelinesLoad = _lib.lookupFunction<_c_guidelines_load, _c_guidelines_load>('panther_guidelines_load');
    _guidelinesBuildEmbeddings = _lib.lookupFunction<_c_guidelines_embed_build, _c_guidelines_embed_build>('panther_guidelines_embeddings_build');
=======
>>>>>>> origin/main
  }

  int init() => _init();

  String generate(String prompt) {
    final cPrompt = prompt.toNativeUtf8(allocator: pkg_ffi.malloc);
    final ptr = _generate(cPrompt.cast());
    pkg_ffi.malloc.free(cPrompt);
    final result = ptr.cast<pkg_ffi.Utf8>().toDartString();
    _free(ptr);
    return result;
  }

  double metricsBleu(String reference, String candidate) {
    final cRef = reference.toNativeUtf8(allocator: pkg_ffi.malloc);
    final cCand = candidate.toNativeUtf8(allocator: pkg_ffi.malloc);
    final score = _bleu(cRef.cast(), cCand.cast());
    pkg_ffi.malloc.free(cRef);
    pkg_ffi.malloc.free(cCand);
    return score;
  }

  double metricsPlagiarism(List<String> corpus, String candidate) {
    final corpusJson = '[${corpus.map((s) => '"${s.replaceAll('"', '\\"')}"').join(',')}]';
    final cJson = corpusJson.toNativeUtf8(allocator: pkg_ffi.malloc);
    final cCand = candidate.toNativeUtf8(allocator: pkg_ffi.malloc);
    final score = _plag(cJson.cast(), cCand.cast());
    pkg_ffi.malloc.free(cJson);
    pkg_ffi.malloc.free(cCand);
    return score;
  }

  double metricsPlagiarismNgram(List<String> corpus, String candidate, int ngram) {
    final corpusJson = '[${corpus.map((s) => '"${s.replaceAll('"', '\\"')}"').join(',')}]';
    final cJson = corpusJson.toNativeUtf8(allocator: pkg_ffi.malloc);
    final cCand = candidate.toNativeUtf8(allocator: pkg_ffi.malloc);
    final n = ngram > 0 ? ngram : 3;
    final score = _plagNgram(cJson.cast(), cCand.cast(), n);
    pkg_ffi.malloc.free(cJson);
    pkg_ffi.malloc.free(cCand);
    return score;
  }

  String validate(String prompt) {
    final cPrompt = prompt.toNativeUtf8(allocator: pkg_ffi.malloc);
    final ptr = _validate(cPrompt.cast());
    pkg_ffi.malloc.free(cPrompt);
    final result = ptr.cast<pkg_ffi.Utf8>().toDartString();
    _free(ptr);
    return result;
  }

  String validateMulti(String prompt, String providersJson) {
    final cPrompt = prompt.toNativeUtf8(allocator: pkg_ffi.malloc);
    final cJson = providersJson.toNativeUtf8(allocator: pkg_ffi.malloc);
    final ptr = _validateMulti(cPrompt.cast(), cJson.cast());
    pkg_ffi.malloc.free(cPrompt);
    pkg_ffi.malloc.free(cJson);
    final result = ptr.cast<pkg_ffi.Utf8>().toDartString();
    _free(ptr);
    return result;
  }

  String validateMultiWithProof(String prompt, String providersJson) {
    final cPrompt = prompt.toNativeUtf8(allocator: pkg_ffi.malloc);
    final cJson = providersJson.toNativeUtf8(allocator: pkg_ffi.malloc);
    final ptr = _validateMultiWithProof(cPrompt.cast(), cJson.cast());
    pkg_ffi.malloc.free(cPrompt);
    pkg_ffi.malloc.free(cJson);
    final result = ptr.cast<pkg_ffi.Utf8>().toDartString();
    _free(ptr);
    return result;
  }

  String validateCustomWithProof(String prompt, String providersJson, String guidelinesJson) {
    final p = prompt.toNativeUtf8(allocator: pkg_ffi.malloc);
    final j = providersJson.toNativeUtf8(allocator: pkg_ffi.malloc);
    final g = guidelinesJson.toNativeUtf8(allocator: pkg_ffi.malloc);
    final ptr = _validateCustomWithProof(p.cast(), j.cast(), g.cast());
    pkg_ffi.malloc.free(p); pkg_ffi.malloc.free(j); pkg_ffi.malloc.free(g);
    final result = ptr.cast<pkg_ffi.Utf8>().toDartString();
    _free(ptr);
    return result;
  }

  String version() {
    final ptr = _version();
    final v = ptr.cast<pkg_ffi.Utf8>().toDartString();
    _free(ptr);
    return v;
  }

  String biasDetect(List<String> samples) {
    final json = '[${samples.map((s) => '"${s.replaceAll('"', '\\"')}"').join(',')}]';
    final cJson = json.toNativeUtf8(allocator: pkg_ffi.malloc);
    final ptr = _biasDetect(cJson.cast());
    pkg_ffi.malloc.free(cJson);
    final result = ptr.cast<pkg_ffi.Utf8>().toDartString();
    _free(ptr);
    return result;
  }

  int tokenCount(String text) {
    final cText = text.toNativeUtf8(allocator: pkg_ffi.malloc);
    final n = _tokenCount(cText.cast());
    pkg_ffi.malloc.free(cText);
    return n;
  }

  double calculateCost(int tokensIn, int tokensOut, String providerName, String costRulesJson) {
    final p = providerName.toNativeUtf8(allocator: pkg_ffi.malloc);
    final r = costRulesJson.toNativeUtf8(allocator: pkg_ffi.malloc);
    final cost = _calculateCost(tokensIn, tokensOut, p.cast(), r.cast());
    pkg_ffi.malloc.free(p);
    pkg_ffi.malloc.free(r);
    return cost;
  }

  String validateOpenAI(String prompt, String apiKey, String model, String base) {
    final p = prompt.toNativeUtf8(allocator: pkg_ffi.malloc);
    final k = apiKey.toNativeUtf8(allocator: pkg_ffi.malloc);
    final m = model.toNativeUtf8(allocator: pkg_ffi.malloc);
    final b = base.toNativeUtf8(allocator: pkg_ffi.malloc);
    final ptr = _validateOpenAI(p.cast(), k.cast(), m.cast(), b.cast());
    pkg_ffi.malloc.free(p); pkg_ffi.malloc.free(k); pkg_ffi.malloc.free(m); pkg_ffi.malloc.free(b);
    final result = ptr.cast<pkg_ffi.Utf8>().toDartString();
    _free(ptr);
    return result;
  }

  String validateOllama(String prompt, String base, String model) {
    final p = prompt.toNativeUtf8(allocator: pkg_ffi.malloc);
    final b = base.toNativeUtf8(allocator: pkg_ffi.malloc);
    final m = model.toNativeUtf8(allocator: pkg_ffi.malloc);
    final ptr = _validateOllama(p.cast(), b.cast(), m.cast());
    pkg_ffi.malloc.free(p); pkg_ffi.malloc.free(b); pkg_ffi.malloc.free(m);
    final result = ptr.cast<pkg_ffi.Utf8>().toDartString();
    _free(ptr);
    return result;
  }

  String validateCustom(String prompt, String providersJson, String guidelinesJson) {
    final p = prompt.toNativeUtf8(allocator: pkg_ffi.malloc);
    final j = providersJson.toNativeUtf8(allocator: pkg_ffi.malloc);
    final g = guidelinesJson.toNativeUtf8(allocator: pkg_ffi.malloc);
    final ptr = _validateCustom(p.cast(), j.cast(), g.cast());
    pkg_ffi.malloc.free(p); pkg_ffi.malloc.free(j); pkg_ffi.malloc.free(g);
    final result = ptr.cast<pkg_ffi.Utf8>().toDartString();
    _free(ptr);
    return result;
  }
<<<<<<< HEAD

  // --- Guidelines ---
  int guidelinesIngest(String json) {
    final j = json.toNativeUtf8(allocator: pkg_ffi.malloc);
    final n = _guidelinesIngest(j.cast());
    pkg_ffi.malloc.free(j);
    return n;
  }
  String guidelinesScores(String query, int topK, String method) {
    final q = query.toNativeUtf8(allocator: pkg_ffi.malloc);
    final m = method.toNativeUtf8(allocator: pkg_ffi.malloc);
    final ptr = _guidelinesSimilarity(q.cast(), topK, m.cast());
    pkg_ffi.malloc.free(q); pkg_ffi.malloc.free(m);
    final s = ptr.cast<pkg_ffi.Utf8>().toDartString();
    _free(ptr);
    return s;
  }
  int guidelinesSave(String name, String json) {
    final n = name.toNativeUtf8(allocator: pkg_ffi.malloc);
    final j = json.toNativeUtf8(allocator: pkg_ffi.malloc);
    final rc = _guidelinesSave(n.cast(), j.cast());
    pkg_ffi.malloc.free(n); pkg_ffi.malloc.free(j);
    return rc;
  }
  int guidelinesLoad(String name) {
    final n = name.toNativeUtf8(allocator: pkg_ffi.malloc);
    final cnt = _guidelinesLoad(n.cast());
    pkg_ffi.malloc.free(n);
    return cnt;
  }
  int guidelinesBuildEmbeddings(String method) {
    final m = method.toNativeUtf8(allocator: pkg_ffi.malloc);
    final rc = _guidelinesBuildEmbeddings(m.cast());
    pkg_ffi.malloc.free(m);
    return rc;
  }
=======
>>>>>>> origin/main
}
