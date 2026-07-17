# Installing Voxta

Voxta is distributed through GitHub Releases.

## macOS

1. Open the latest Voxta release on GitHub.
2. Download the macOS `.dmg` attached to the release.
3. Verify the checksum if a `SHA256SUMS-macos.txt` file is attached.
4. Open the `.dmg`.
5. Drag `Voxta.app` into `Applications`.
6. Open Voxta from `Applications`.
7. Grant Microphone, Accessibility, and Input Monitoring permissions when prompted.
8. Quit and reopen Voxta after granting Accessibility or Input Monitoring permissions.
9. Download one Whisper model from Voxta settings.

After the model download finishes, normal dictation works offline.

## Windows

1. Open the latest Voxta release on GitHub.
2. Download the Windows `.msi` installer attached to the release.
3. Verify the checksum if a `SHA256SUMS-windows.txt` file is attached.
4. Run the installer.
5. Start Voxta from the Start menu.
6. Select a microphone and download one Whisper model from Voxta settings.

Voxta should not be run as Administrator for normal use. Dictating into applications that are running as Administrator may be blocked by Windows input isolation.

## First Dictation

1. Put the cursor in a text field in another app.
2. Use the configured shortcut.
3. Speak.
4. Stop recording.
5. Voxta inserts the final local transcription.

Default shortcut is `F8`, and default shortcut behavior is `Hold to talk`. You can switch to `Press once to start, press again to stop` in Settings.

## Privacy During Setup

Voxta requires network access during setup only to download a Whisper model. Optional AI cleanup can make additional requests only if the user enables it.

Core dictation does not require:

- An account.
- API keys.
- Ollama.
- Cloud transcription.
- Internet access after model download.
