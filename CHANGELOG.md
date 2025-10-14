Changelog

0.1.1 — Stage 1 (Offline Proofs) and Packaging
- Added proof module (Stage 1) with SHA3-512 hashing and canonical JSON:
  - `compute_proof`, `verify_proof_local` in `crates/panther-validation`.
- New FFI exports (feature `validation`):
  - `panther_validation_run_multi_with_proof` → returns `{results, proof}`.
  - `panther_proof_compute` → computes proof from inputs/results.
- Python API extensions:
  - `POST /proof/compute` endpoint (uses FFI, Python fallback).
  - Improved dynamic loading of FFI across platforms.
- Samples updated to show proof hash (combined_hash):
  - Swift/Kotlin/Flutter/React Native now support “validate with proof” and display the hash.
- Packaging scripts:
  - New scripts in `scripts/release/` (iOS xcframework, Android AAR, Python wheels, WASM, headers, manifest/checksums).
- `panther_version_string` is dynamic (from crate version) and samples expose version helpers.

0.1.0 — Initial SDK
- Core Engine, FFI, Validation (OpenAI/Ollama), Python API, Samples, Storage (in-memory/sled), Metrics/Logs.
