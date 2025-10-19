Flutter Sample (Android Emulator / iOS Simulator)

Goal: Call PantherSDK via Dart FFI.

What this sample does
- Validate (white‑label): UI fields for provider (type/base/model/key) and a button that calls `validateMultiWithProof(prompt, providersJson)` via Dart FFI.
- Proof: displays `combined_hash` and buttons to Anchor/Check Status via HTTP (backend Python API).
- Metrics: plagiarism (Jaccard n‑gram) via FFI.
- Token + Cost estimate: each result line appends `tokens_in/tokens_out` and `$cost` using `panther_token_count` and `panther_calculate_cost` with an embedded pricing table.

1) Build native libs
- Android (x86_64 emulator):
  `cargo ndk -t x86_64 -o android/app/src/main/jniLibs build -p panther-ffi --release --features "validation validation-openai validation-ollama metrics-inmemory"`
- iOS Simulator (arm64):
  `cargo build -p panther-ffi --release --target aarch64-apple-ios-sim --features "validation validation-openai validation-ollama metrics-inmemory"`
  Add `libpanther_ffi.a` to `ios/Runner` target in Xcode and include `panther.h`.

2) Persisting cost rules
- Add dependency: `flutter pub add shared_preferences`
- iOS: ensure CocoaPods installs the plugin in `ios/`.

3) Dart FFI bindings
- See `samples/flutter/lib/ffi.dart` to bind `panther_init`, `panther_generate`, `panther_free_string`.
- Add dependency: `flutter pub add ffi` in the generated app directory.

Example (Dart) — building providers JSON
```dart
final providers = [
  {
    'type': 'openai',
    'base_url': 'https://api.openai.com',
    'model': 'gpt-4o-mini',
    'api_key': 'sk-...'
  }
];
final json = providers.toString().replaceAll("'", '"');
final out = panther.validateMulti('Explain insulin function', json);
```

4) App code
- See `samples/flutter/lib/main.dart` to call FFI and display the result.

5) Run
- Android emulator: `flutter run -d emulator-5554`
- iOS Simulator: open `ios` in Xcode, ensure lib linked, run.

Creation
- Run `samples/flutter/GENERATE_PROJECT.sh` to create `PantherSample` Flutter app and copy sample sources.
