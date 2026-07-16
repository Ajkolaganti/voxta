# Contributing

Thanks for helping improve Voxta.

## Development Setup

Install:

- Node.js 18+
- npm 10+
- Rust stable
- CMake
- Platform build tools for macOS or Windows

Then run:

```bash
npm ci
npm run typecheck
npm run lint
npm test
cargo test --workspace
```

If CMake is not installed, you can still run core Rust tests without the whisper.cpp feature:

```bash
cargo test --workspace --no-default-features
```

## Pull Requests

- Keep changes focused.
- Do not add telemetry, accounts, cloud transcription, transcript history, or LLM rewriting.
- Add or update tests for behavior changes.
- Update docs when user-visible behavior changes.
- Preserve clipboard and privacy guarantees.

## Code Style

- TypeScript: ESLint and TypeScript strict mode.
- Rust: `cargo fmt` and Clippy with warnings denied.
- Prefer deterministic local processing over hidden network behavior.

## Native Platform Work

Platform-specific code should stay behind traits in `src-tauri/src/platform`, `focus`, `hotkeys`, and `text_injection`.
