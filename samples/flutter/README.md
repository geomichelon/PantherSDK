Flutter Sample (Android Emulator / iOS Simulator)

Goal: Call PantherSDK via Dart FFI.

What this sample does
- Generate: calls `panther_generate` and displays JSON output
- BLEU: computes BLEU-1 for Reference vs Output via `panther_metrics_bleu`
  (You can extend to record metrics and get logs using `panther_metrics_record`, `panther_storage_list_metrics`, `panther_logs_get`.)
- Validate (white‑label): UI fields for provider (type/base/model/key) and a button `Validate (multi)` that calls `validateMulti(prompt, providersJson)` via Dart FFI.

1) Build native libs
- Android (x86_64 emulator):
  `cargo ndk -t x86_64 -o android/app/src/main/jniLibs build -p panther-ffi --release`
- iOS Simulator (arm64):
  `cargo build -p panther-ffi --release --target aarch64-apple-ios-sim`
  Add `libpanther_ffi.a` to `ios/Runner` target in Xcode and include `panther.h`.

2) Dart FFI bindings
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

3) App code
- See `samples/flutter/lib/main.dart` to call FFI and display the result.

4) Run
- Android emulator: `flutter run -d emulator-5554`
- iOS Simulator: open `ios` in Xcode, ensure lib linked, run.

Creation
- Run `samples/flutter/GENERATE_PROJECT.sh` to create `PantherSample` Flutter app and copy sample sources.
