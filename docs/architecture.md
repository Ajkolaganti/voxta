# Architecture

Voxta is a layered Tauri 2 desktop app.

## Layers

Frontend:

- `src/pages`: settings, onboarding, overlay.
- `src/components`: reusable controls.
- `src/services`: typed Tauri command client.
- `src/types`: shared TypeScript types.

Backend:

- `src-tauri/src/audio`: microphone capture, audio buffers, resampling.
- `src-tauri/src/config`: config persistence and validation.
- `src-tauri/src/focus`: focused-app snapshots.
- `src-tauri/src/hotkeys`: press-and-hold global shortcut state.
- `src-tauri/src/models`: model metadata, downloads, checksums, deletion.
- `src-tauri/src/permissions`: permission status model.
- `src-tauri/src/platform`: macOS, Windows, unsupported platform adapters.
- `src-tauri/src/runtime`: dictation state orchestration.
- `src-tauri/src/text_injection`: direct insertion and clipboard fallback.
- `src-tauri/src/transcription`: transcript cleanup and whisper.cpp engine.
- `src-tauri/src/tray`: menu-bar/system-tray controls.

## Runtime Flow

1. Global shortcut key-down sends `ShortcutEvent::Start`.
2. Runtime moves `Idle -> Recording`.
3. Focus snapshot is captured.
4. Audio worker starts a `cpal` microphone stream.
5. Overlay shows `Listening...`.
6. Shortcut key-up sends `ShortcutEvent::Stop`.
7. Audio worker stops and returns an in-memory audio buffer.
8. Empty or silent recordings are discarded.
9. Runtime moves `Recording -> Transcribing`.
10. `WhisperCppEngine` transcribes with the selected local model.
11. Deterministic cleanup is applied.
12. Runtime moves `Transcribing -> Inserting`.
13. Focus is restored where supported.
14. Text is inserted using direct mode or clipboard fallback.
15. Runtime returns to `Idle`.

Escape during recording sends `ShortcutEvent::Cancel`, drops the in-memory recording, and returns to `Idle`.

## State Machine

Valid transitions:

- `Idle -> Recording`
- `Recording -> Transcribing`
- `Transcribing -> Inserting`
- `Inserting -> Idle`
- `Recording -> Cancelled -> Idle`
- `Any state -> Error -> Idle`

The state machine rejects concurrent recordings.

## Native Boundaries

Native platform code is isolated behind traits and platform modules:

- macOS: Accessibility checks, System Settings links, AppleScript focus best effort, system sounds.
- Windows: foreground-window snapshot, Windows Settings links, system sounds.
- Unsupported platforms: no-op or explicit errors.

Linux support should add a new platform implementation without changing frontend contracts.
