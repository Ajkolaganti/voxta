# Troubleshooting

## The shortcut does nothing

- Confirm Voxta is not paused in the tray/menu-bar menu.
- Confirm the configured shortcut has at least one modifier and one non-modifier key.
- On macOS, confirm Accessibility permission is enabled.
- Try changing the shortcut in Settings.

If local logs show `EventTapError`, macOS is blocking global keyboard monitoring. Open System Settings -> Privacy & Security -> Accessibility, enable Voxta, then quit and reopen the app. Unsigned local builds may need to be removed and added again after rebuilding.

## `npm run tauri:dev` fails with `failed to get cargo metadata`

The Tauri CLI could not find Cargo. Install Rust, then open a new terminal:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
cargo --version
```

Then rerun:

```bash
npm run tauri:dev
```

## The app cannot hear the microphone

- Confirm microphone permission.
- Select a different microphone in Settings.
- Use the Test microphone button.
- Confirm another app is not exclusively using the device.

## Text is not inserted

- Try Clipboard paste fallback mode.
- Confirm the target field is editable.
- On Windows, avoid elevated target applications.
- On macOS, confirm Accessibility permission.
- Password fields may be refused intentionally.

## Transcription is unavailable

- Confirm a model is downloaded and selected.
- Confirm the model checksum passes.
- If building from source, install CMake and build with the default `whisper` feature.

## Live preview does not appear

- Confirm Enable live transcription preview is on in Settings.
- Use the Tiny or Base model and Fast streaming quality if the selected model is too slow.
- The app may skip preview chunks to avoid queueing work; final transcription should still run after release.
- If the overlay says live preview is unavailable, continue testing final insertion first.
- Live preview needs enough speech context to produce stable text. Short one- or two-word recordings may only appear after final transcription.

## I do not want to hold the shortcut while talking

Open Settings -> Shortcut -> Shortcut behavior and choose `Press once to start, press again to stop`.
Hold mode remains the default because it is less likely to leave recording enabled by accident.

## Voice commands are typed literally

- Confirm Enable voice commands is on.
- In the default Explicit prefix mode, say the configured prefix before the command, for example `Voxta new line`.
- Use the Test command parser in Settings to confirm the phrase is recognized.
- Natural-phrase mode recognizes commands without a prefix and can be easier to trigger accidentally.

## AI cleanup does not run

- Confirm AI cleanup is enabled.
- Confirm the provider is not None.
- For Ollama, confirm Ollama is running and that the configured model is installed.
- For an OpenAI-compatible endpoint, confirm the base URL, model, API key, and timeout.
- Cleanup failures do not block dictation; Voxta inserts the original local transcription and shows a warning.

## Build fails on whisper-rs-sys

Install CMake.

macOS:

```bash
brew install cmake
```

Windows:

```powershell
choco install cmake
```

Then rerun:

```bash
cargo test --workspace
```

## DMG packaging fails with `Device not configured`

The optimized app may build successfully while DMG creation fails if the local environment cannot create disk images:

```text
hdiutil: create failed - Device not configured
```

Run the release workflow on a normal macOS runner or a local macOS machine with working `hdiutil`.
