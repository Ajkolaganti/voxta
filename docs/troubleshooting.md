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
