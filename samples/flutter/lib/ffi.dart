import 'dart:ffi' as ffi;
import 'dart:io';
import 'package:ffi/ffi.dart' as pkg_ffi;

typedef _c_init = ffi.Int32 Function();
typedef _c_generate = ffi.Pointer<ffi.Char> Function(ffi.Pointer<ffi.Char>);
typedef _c_free = ffi.Void Function(ffi.Pointer<ffi.Char>);
typedef _c_metrics_bleu = ffi.Double Function(ffi.Pointer<ffi.Char>, ffi.Pointer<ffi.Char>);
typedef _c_validate = ffi.Pointer<ffi.Char> Function(ffi.Pointer<ffi.Char>);
typedef _c_validate_multi = ffi.Pointer<ffi.Char> Function(ffi.Pointer<ffi.Char>, ffi.Pointer<ffi.Char>);
typedef _c_version = ffi.Pointer<ffi.Char> Function();
typedef _c_validate_multi_with_proof = ffi.Pointer<ffi.Char> Function(ffi.Pointer<ffi.Char>, ffi.Pointer<ffi.Char>);

class PantherFFI {
  late final ffi.DynamicLibrary _lib;
  late final _c_init _init;
  late final _c_generate _generate;
  late final _c_free _free;
  late final _c_metrics_bleu _bleu;
  late final _c_validate _validate;
  late final _c_validate_multi _validateMulti;
  late final _c_version _version;
  late final _c_validate_multi_with_proof _validateMultiWithProof;

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
    _validate = _lib.lookupFunction<_c_validate, _c_validate>('panther_validation_run_default');
    _validateMulti = _lib.lookupFunction<_c_validate_multi, _c_validate_multi>('panther_validation_run_multi');
    _version = _lib.lookupFunction<_c_version, _c_version>('panther_version_string');
    _validateMultiWithProof = _lib.lookupFunction<_c_validate_multi_with_proof, _c_validate_multi_with_proof>('panther_validation_run_multi_with_proof');
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

  String version() {
    final ptr = _version();
    final v = ptr.cast<pkg_ffi.Utf8>().toDartString();
    _free(ptr);
    return v;
  }
}
