Contributing Guidelines

Language
- Write all code comments and documentation in English.

Architecture
- Follow the hexagonal architecture: keep domain, application (use cases), and adapters separated.
- Do not let interface/FFI concerns leak into domain or core crates.
- Depend inward: `panther-ffi` depends on `panther-core` and its dependencies, never the opposite.

Testing
- Each Rust crate must be isolated and testable without the interface layer.
- Provide unit tests under each crate verifying behavior via traits and mock implementations.
- Avoid global state in tests; prefer dependency injection and local mocks.

Dependencies
- Keep core/domain dependencies minimal. Use features for optional providers/integrations.
- Prefer small, well-known crates; justify new dependencies in PRs.

Style
- Rust 2021 edition, stable toolchain. Keep functions small and focused.

