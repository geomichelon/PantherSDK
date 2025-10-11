Bindings and Generation

C Header (via cbindgen)
- Generate the header: `cbindgen --crate panther-ffi --output bindings/include/panther.h`
- Exposed functions (core): `panther_init`, `panther_version_string`, `panther_generate`, `panther_free_string`
- Validation (white‑label):
  - `panther_validation_run_default(prompt)`
  - `panther_validation_run_openai(prompt, api_key, model, base)`
  - `panther_validation_run_ollama(prompt, base, model)`
  - `panther_validation_run_multi(prompt, providers_json)` where `providers_json` is a JSON array like:
    `[{"type":"openai","api_key":"sk-...","base_url":"https://api.openai.com","model":"gpt-4o-mini"}]`

iOS (Swift)
- Add `panther.h` to the bridging header and link `libpanther_ffi`
- Write a Swift wrapper to convert String <-> C char*

Android (Kotlin/JNI)
- Use the NDK to load `libpanther_ffi.so`
- Create JNI functions that call the C symbols

Flutter (Dart FFI)
- Use `dart:ffi` to map the C ABI functions

React Native (TurboModule/JSI)
- Implement a C++ TurboModule that calls the C ABI on each platform

Note: Generators like UniFFI (Swift/Kotlin) and flutter_rust_bridge (Dart) can reduce manual code. RN still requires TurboModule/JSI bridging.
