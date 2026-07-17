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
- `src-tauri/src/hotkeys`: hold-to-talk and toggle global shortcut state.
- `src-tauri/src/models`: model metadata, downloads, checksums, deletion.
- `src-tauri/src/permissions`: permission status model.
- `src-tauri/src/platform`: macOS, Windows, unsupported platform adapters.
- `src-tauri/src/runtime`: dictation state orchestration.
- `src-tauri/src/streaming`: live preview chunking, overlap, and stale-session safeguards.
- `src-tauri/src/text_injection`: direct insertion and clipboard fallback.
- `src-tauri/src/transcription`: transcript cleanup and whisper.cpp engine.
- `src-tauri/src/tray`: menu-bar/system-tray controls.
- `src-tauri/src/voice_commands`: deterministic command parsing, execution, and history.
- `src-tauri/src/ai_cleanup`: optional cleanup providers and secure-key access.

## Runtime Flow

1. Global shortcut key-down sends `ShortcutEvent::Start` in hold mode, or toggles recording in toggle mode.
2. Runtime moves `Idle -> Recording`.
3. Focus snapshot is captured.
4. Audio worker starts a `cpal` microphone stream.
5. Overlay shows `Listening...`.
6. If enabled, streaming preview periodically snapshots the in-memory recording, transcribes a bounded overlapping chunk, and updates only the overlay.
7. Shortcut key-up sends `ShortcutEvent::Stop` in hold mode; toggle mode stops on the next shortcut key-down.
8. Streaming preview is cancelled and stale preview results are ignored.
9. Audio worker stops and returns the complete in-memory audio buffer.
10. Empty or silent recordings are discarded.
11. Runtime moves `Recording -> Transcribing`.
12. `WhisperCppEngine` transcribes the complete recording with the selected local model.
13. Runtime moves `Transcribing -> ApplyingCommands` and deterministic voice commands are applied to the current dictation buffer.
14. If enabled, runtime moves to `CleaningUp` and sends the command-processed final text to the selected cleanup provider.
15. Cleanup failures fall back to the original local transcription.
16. Runtime moves to `Inserting`.
17. Focus is restored where supported.
18. Text is inserted using direct mode or clipboard fallback.
19. Runtime returns to `Idle`.

Escape during recording sends `ShortcutEvent::Cancel`, drops the in-memory recording, and returns to `Idle`.

## State Machine

Valid transitions:

- `Idle -> Recording`
- `Recording -> Transcribing`
- `Transcribing -> ApplyingCommands`
- `ApplyingCommands -> CleaningUp`
- `ApplyingCommands -> Inserting`
- `CleaningUp -> Inserting`
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
