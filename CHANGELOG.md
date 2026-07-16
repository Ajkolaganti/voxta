# Changelog

## 0.2.0 - Smart Dictation Preview

Preview release for optional smart dictation features. The core push-to-talk local dictation workflow remains available without Ollama, API keys, accounts, or internet after model download.

- Added local live transcription preview while recording, with configurable interval and quality.
- Added deterministic voice commands with explicit-prefix mode by default.
- Added optional AI cleanup provider abstraction with None, Ollama, and OpenAI-compatible endpoint options.
- Added Smart Dictation settings and privacy indicators.
- Added tests for streaming helpers, voice command parsing/execution, and cleanup prompt behavior.
- Updated overlay states for Listening, Transcribing, Applying commands, Cleaning up, and Inserting.

## 0.1.0 - Early Preview

Initial public preview release. This release is intended for source publication and cross-platform manual QA before production claims.

- Initial MVP source structure.
- Tauri 2 desktop app scaffold.
- React settings and onboarding UI.
- Tray/menu-bar behavior.
- Press-and-hold shortcut state handling.
- In-memory audio capture and 16 kHz mono conversion.
- whisper.cpp transcription integration behind the default `whisper` feature.
- Model download, checksum verification, and deletion.
- Text insertion with direct and clipboard fallback modes.
- Privacy, security, release, and platform documentation.
