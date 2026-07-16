# Voxta

Voxta — Private voice typing that works everywhere.

Hold a global keyboard shortcut, speak, release the shortcut, and Voxta transcribes locally with whisper.cpp and inserts the text into the app that already had focus.

Voxta does not include accounts, telemetry, analytics, cloud transcription, transcript history, collaboration, payments, or chat.

Voxta does not require Ollama, API keys, or a user account. It uses a local Whisper model that you download once during setup; after that, dictation works offline. Optional AI cleanup is disabled by default.

## Status

Version: `0.2.0`

Release label: **Early Preview**

This is an early preview source release candidate. It is not described as production-proven until macOS and Windows manual QA has been completed on signed release builds. Full local transcription builds require CMake because `whisper-rs` builds whisper.cpp native code.

## Supported Platforms

- macOS Apple Silicon: primary target
- Windows 10 and Windows 11: primary target
- macOS Intel: practical target
- Linux: code is structured for later support, but Linux is not part of the early preview release

Windows builds are covered by the GitHub Actions workflow configuration, but Windows runtime behavior has not been manually tested from this macOS development environment.

## Screenshots

Screenshots will be added after the first signed release build.

## Installation

For non-technical users, download the latest installer from GitHub Releases once maintainers publish signed binaries:

- macOS: `.dmg`
- Windows: `.msi`

Do not download Voxta installers from unofficial sources.

## How To Use

1. Start Voxta.
2. Complete onboarding.
3. Download a Whisper model.
4. Put your cursor in any editable field.
5. Hold the configured shortcut.
6. Speak.
7. Release the shortcut.
8. Voxta inserts the transcription at the cursor.

Default shortcut: `Ctrl+Alt+Space`.

You can change the shortcut in Settings.

## Smart Dictation Preview

Voxta 0.2.0 adds optional quality-of-life features:

- Live transcription preview while holding the shortcut. This preview is local and is not inserted into the target app.
- Deterministic voice commands such as `Voxta new line`, `Voxta delete last word`, `Voxta undo`, and `Voxta cancel dictation`.
- Optional AI cleanup after final local transcription. This is off by default.

The reliable core workflow remains unchanged: hold shortcut, speak, release, local final transcription, insert final text.

AI cleanup providers:

- None: default; no cleanup is applied.
- Ollama: uses a locally running Ollama server, usually `http://127.0.0.1:11434`.
- OpenAI-compatible endpoint: advanced option; sends final dictated text to the configured endpoint.

No API key is required unless you enable a remote endpoint that requires one. API keys are saved through operating-system credential storage, not the plain-text Voxta config file.

## Permissions

macOS requires:

- Microphone access for recording speech.
- Accessibility access for global keyboard monitoring and cross-app insertion.

Windows requires:

- Microphone permission.
- Normal user-level input permissions. Voxta should not run as Administrator.

Voxta refuses secure-field insertion when the operating system exposes enough information to identify the field. Some apps do not expose this metadata consistently.

## Models

Voxta downloads ggml Whisper models from the documented `ggerganov/whisper.cpp` Hugging Face repository.

Available defaults:

- Tiny multilingual: 75 MiB, fastest
- Tiny English: 75 MiB, faster English-only mode
- Base multilingual: 142 MiB, default balance
- Base English: 142 MiB, faster English-only mode
- Small multilingual: 466 MiB, better accuracy
- Small English: 466 MiB, better English-only accuracy

Downloaded model files are verified against the upstream SHA checksums before use and stored in the per-user application data directory.

## Privacy

Voxta:

- Transcribes speech locally.
- Does not send microphone audio to a cloud speech API.
- Does not require an API key.
- Does not create accounts.
- Does not store audio or transcript history by default.
- Does not include analytics or telemetry SDKs.
- Does not permanently overwrite the clipboard.
- Keeps live preview and voice commands local.
- Sends text to a remote cleanup endpoint only if AI cleanup is enabled and a remote provider is selected.

Network access is used for model downloads, optional Ollama or OpenAI-compatible cleanup requests when enabled, and user-opened external links. Automatic updates are not implemented in this MVP.

See [PRIVACY.md](./PRIVACY.md) for details.

## Build From Source

Prerequisites:

- Node.js 18 or newer
- npm 10 or newer
- Rust stable
- CMake
- macOS: Xcode Command Line Tools
- Windows: Visual Studio Build Tools with C++ workload

Install dependencies:

```bash
npm ci
```

Run the app in development:

```bash
npm run tauri:dev
```

Build installers:

```bash
npm run tauri:build
```

Validation commands for maintainers:

```bash
npm ci
npm run typecheck
npm run lint
npm test
npm run build
npm audit --json
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

The Rust commands require CMake on `PATH`.

Installer build command:

```bash
npm run tauri:build
```

If a local macOS environment cannot create disk images, `npm run tauri:build` may compile the optimized app and produce `Voxta.app`, then fail during DMG packaging with `hdiutil: create failed - Device not configured`. Treat that as an environment packaging failure only after confirming the `.app` bundle and release executable exist.

## Architecture

Voxta uses:

- Tauri 2 for the desktop shell.
- Rust for native audio, shortcut, model, focus, and insertion services.
- React + TypeScript + Vite for settings and onboarding.
- whisper.cpp through `whisper-rs` for local speech-to-text.

Main backend traits:

- `AudioRecorder`
- `TranscriptionEngine`
- `TextInjector`
- `FocusedApplicationTracker`
- `GlobalShortcutManager`
- `ModelManager`

See [docs/architecture.md](./docs/architecture.md).

## Known Limitations

- Full local transcription builds require CMake.
- macOS and Windows key-up behavior must be runtime-tested on signed app builds.
- Clipboard fallback preserves text clipboard contents; rich clipboard formats may not be preserved on every platform.
- Secure-field detection depends on operating-system and target-application metadata.
- Live preview is best-effort and may skip chunks on slow machines; final transcription remains authoritative.
- AI cleanup is preview functionality and falls back to the original transcript on failure.
- Binaries are not claimed to be signed until maintainers configure real signing credentials.

## App Icons

This repository includes temporary Voxta icons generated for the Early Preview release. They use a simple microphone/sound-wave concept and live in `src-tauri/icons/`.

Maintainers can replace:

- `src-tauri/icons/32x32.png`
- `src-tauri/icons/128x128.png`
- `src-tauri/icons/128x128@2x.png`
- `src-tauri/icons/icon.icns`
- `src-tauri/icons/icon.ico`

Keep tray/menu-bar icon variants legible at small sizes and transparent where the platform expects transparency.

## Release Verification

Each release should include SHA-256 checksums for installer artifacts. Verify a downloaded release before installing:

```bash
shasum -a 256 Voxta_*.dmg
```

On Windows PowerShell:

```powershell
Get-FileHash .\Voxta_*.msi -Algorithm SHA256
```

Compare the output with the checksums attached to the GitHub Release.

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md).

## Security

Report security issues privately. See [SECURITY.md](./SECURITY.md).

## Acknowledgments

Voxta builds on Tauri, React, Vite, Rust, whisper.cpp, OpenAI Whisper model weights, cpal, rdev, enigo, arboard, and the broader open-source ecosystem.
