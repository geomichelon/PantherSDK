PantherSDK Swift Sample — Anthropic Preset

Overview
- This sample demonstrates Panther validation (single/multi/with proof) with provider presets. In addition to OpenAI and Ollama, an Anthropic preset is available.

Anthropic Preset
- UI fields: Anthropic API Key, Base URL (default https://api.anthropic.com), and model (e.g., claude-3-5-sonnet-latest).
- Modes:
  - Single: under Provider = Anthropic, the sample builds a single-entry providers JSON and runs validation.
  - Multi/With Proof: Provider selection contributes an Anthropic entry in the providers array; proof hash is shown in "With Proof".

Providers JSON (example)
```
[
  {"type": "anthropic", "api_key": "sk-...", "base_url": "https://api.anthropic.com", "model": "claude-3-5-sonnet-latest"}
]
```

Notes
- The Core/FFI must be built with Anthropic features enabled to run Anthropic calls:
  - Sync: `-p panther-ffi --features "validation validation-anthropic"`
  - Async: `-p panther-ffi --features "validation validation-async validation-anthropic-async"`
- Costs shown are estimates based on the editable rules in the sample.
- For local testing, OpenAI and Ollama presets remain available.

