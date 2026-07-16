# Voxta implementation plan

## Environment findings

- Workspace: repository root
- Existing repository contents: empty at start
- Git repository: not initialized at start
- Node.js: available (`v18.20.4`)
- npm: available (`10.9.0`)
- Rust/Cargo: not installed in the current environment at start

Rust is required to build and run the Tauri backend. The source should still be structured for Tauri 2 and checked in with pinned dependency manifests. Rust build verification requires installing a Rust toolchain or running CI.

## Product goal

Voxta is a privacy-first desktop dictation app:

1. Runs quietly from the tray/menu bar.
2. Starts recording on global shortcut key-down.
3. Stops recording on shortcut key-up.
4. Transcribes locally with whisper.cpp.
5. Inserts text into the application that was focused before recording.
6. Stores no audio or transcript history by default.

## Architecture

Frontend:

- React + TypeScript + Vite.
- Minimal settings and onboarding UI.
- Tauri commands for configuration, model management, permissions, microphone listing, and test dictation.

Backend:

- Tauri 2 application in Rust.
- Layered services with traits:
  - `AudioRecorder`
  - `TranscriptionEngine`
  - `TextInjector`
  - `FocusedApplicationTracker`
  - `GlobalShortcutManager`
  - `ModelManager`
- Recording state machine:
  - `Idle -> Recording -> Transcribing -> Inserting -> Idle`
  - `Recording -> Cancelled -> Idle`
  - `Any -> Error -> Idle`
- Platform modules:
  - `platform::macos`
  - `platform::windows`
  - `platform::unsupported`

## Implementation sequence

1. Create project metadata, documentation, lint/test/build scripts, and Tauri configuration.
2. Implement Rust configuration loading, validation, persistence, and app data paths.
3. Implement recording state machine and shortcut event handling independent of OS backends.
4. Implement transcript cleanup and spoken-command processing.
5. Implement model metadata, checksum verification, download state, and delete support.
6. Implement audio buffer types, microphone device listing, PCM conversion hooks, and empty-recording detection.
7. Implement whisper.cpp engine using `whisper-rs` with CPU default and Metal feature support on macOS builds.
8. Implement macOS focus tracking, Accessibility permission checks, direct insertion, and clipboard fallback.
9. Implement Windows focus tracking, SendInput insertion, UI Automation secure-field detection where available, and clipboard fallback.
10. Implement tray/menu integration, launch-at-login configuration, background startup, and recording overlay window events.
11. Implement React settings, onboarding, status, shortcut recorder UI, model download UI, microphone test, and test dictation field.
12. Add unit tests for state transitions, transcript cleanup, spoken commands, configuration, checksum validation, cancellation, and clipboard restoration interfaces.
13. Add GitHub Actions for lint, format, tests, build checks, release bundles, and checksums.
14. Add public project documentation and release guidance.
15. Run available local checks. If Rust is unavailable, document unverified Rust checks exactly.

## Platform strategy

### macOS first

macOS is the first full platform target because the current environment is macOS-like and Voxta prioritizes Apple Silicon.

Implementation notes:

- Capture global key-down/up with native event monitoring when Tauri global shortcut does not provide reliable release events.
- Request Microphone and Accessibility permissions.
- Avoid focus stealing by saving the previously focused app/window before showing the overlay.
- Prefer direct CGEvent text insertion where practical.
- Use pasteboard fallback with clipboard restore.
- Detect secure input/password fields when Accessibility exposes that information, and refuse dictation.

### Windows second

Implementation notes:

- Use low-level keyboard hook semantics for key-down/up.
- Use SendInput for direct insertion.
- Use clipboard paste fallback with clipboard restore.
- Handle elevated target windows gracefully by surfacing an error instead of requiring Voxta to run as Administrator.
- Refuse known password fields when UI Automation exposes control metadata.

### Linux later

Linux is intentionally isolated behind traits but not implemented for version 0.1.0.

## MVP definition

The 0.1.0 MVP is complete when:

- Settings and onboarding are usable.
- Configuration persists and survives corrupted files.
- Shortcut state handling prevents concurrent recordings.
- Audio recording feeds local transcription.
- Model downloads are verified by checksum and deletable.
- Dictation text is inserted into the previous focused application without permanently overwriting the clipboard.
- Tray/menu bar operation and launch-at-login are available.
- Documentation and CI are ready for GitHub publication.

## Verification plan

Local:

- `npm install`
- `npm run typecheck`
- `npm run lint`
- `npm test`
- `npm run build`

Rust/Tauri where Cargo is available:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `npm run tauri build`

Manual QA:

- Chrome
- VS Code
- Slack
- Microsoft Word
- Notion
- Gmail
- Discord
- Native text editors
- Terminals
- Multiline fields
- Password fields
- Elevated Windows applications

## Known implementation risks

- Global key-up handling is platform-specific and must be validated on real macOS and Windows systems.
- Whisper model download URLs and checksums must be maintained whenever model choices change.
- Code signing and notarization cannot be completed without maintainer-owned credentials.
- Windows secure-field detection may depend on target application UI Automation support.
