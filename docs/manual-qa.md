# Manual QA Checklist

Run this checklist on signed or local release builds before publishing preview builds.

Record OS version, CPU architecture, Voxta version, build type, selected model, insertion mode, and shortcut before testing.

## Setup

- [ ] Fresh install opens onboarding on manual launch.
  Expected: Settings/onboarding appears once, and the tray/menu-bar item is present.
- [ ] App starts hidden when launched with `--background`.
  Expected: no settings window appears, tray/menu-bar item is present.
- [ ] Launch-at-login toggle updates OS startup registration.
  Expected: the OS startup entry appears when enabled and is removed when disabled.
- [ ] Model download shows progress.
  Expected: progress updates until completion and the model is marked installed.
- [ ] Downloaded model checksum verifies.
  Expected: the model becomes selectable only after checksum verification.
- [ ] Deleted model is removed from app data.
  Expected: the model row returns to Download and the model file is gone from the app data directory.

## Dictation Targets

- [ ] Apple Notes.
  Steps: open a note, place the cursor in the body, use the configured shortcut, say a short sentence, stop recording.
  Expected: text appears in the note, focus returns to Notes, and clipboard text is preserved.
- [ ] Chrome text field.
  Steps: open a plain text field or search box, dictate a short sentence.
  Expected: text appears at the cursor without submitting the form.
- [ ] Gmail compose field.
  Steps: open a draft, place the cursor in the message body, dictate two sentences.
  Expected: text appears in the body and the message is not sent.
- [ ] Slack message box.
  Steps: focus a message composer, dictate one sentence.
  Expected: text appears and Enter is not pressed.
- [ ] VS Code editor.
  Steps: focus a source file, dictate code-like words such as "const user equals null".
  Expected: text is inserted without unrelated rewriting.
- [ ] Terminal prompt.
  Steps: focus a shell prompt, dictate a short command-like phrase.
  Expected: text is inserted and the command is not executed automatically.
- [ ] Microsoft Word document.
  Steps: focus a document body, dictate a sentence.
  Expected: text appears in the document and focus remains in Word.
- [ ] Notion page.
  Steps: focus a page block, dictate a sentence.
  Expected: text appears in the current block.
- [ ] Discord message box.
  Steps: focus a message composer, dictate a sentence.
  Expected: text appears and the message is not sent.
- [ ] Native text editor.
  Steps: use TextEdit on macOS or Notepad on Windows and dictate a sentence.
  Expected: text appears at the cursor.
- [ ] Multiline field.
  Steps: dictate "first line new line second line".
  Expected: a line break is inserted only when spoken commands are enabled.
- [ ] Password field.
  Steps: focus a known password field and attempt dictation.
  Expected: Voxta refuses insertion when the operating system exposes secure-field metadata.
- [ ] Windows elevated app.
  Steps: focus an elevated app such as Administrator Notepad and dictate.
  Expected: Voxta shows a clear insertion/focus error and does not ask to run as Administrator.

## Shortcut And State

- [ ] Key-down starts recording.
  Expected: overlay appears immediately and status becomes Listening.
- [ ] Holding does not start multiple recordings.
  Expected: repeated key-down events do not create concurrent recordings.
- [ ] Key-up stops recording.
  Expected: overlay changes to Transcribing and microphone capture stops.
- [ ] Toggle shortcut mode.
  Steps: set Shortcut behavior to `Press once to start, press again to stop`, press the shortcut, speak, release all keys, press the shortcut again.
  Expected: recording continues after the first key release and stops only on the second shortcut press.
- [ ] Escape cancels recording.
  Expected: no text is inserted, audio is discarded, and status returns to Idle.
- [ ] Rapid shortcut presses.
  Expected: invalid concurrent transitions are rejected and the app returns to Idle.
- [ ] Long recording.
  Expected: recording remains stable, transcribes after release, and UI does not freeze.
- [ ] Empty recording.
  Expected: "No speech detected" is shown and no text is inserted.
- [ ] Shortcut can be changed.
  Expected: the new shortcut works after saving and the old shortcut no longer starts recording.
- [ ] Recording indicator does not take focus.
  Expected: text is inserted into the app that was focused before recording.

## Smart Dictation Preview

- [ ] Live preview while recording.
  Steps: enable live preview, start recording, speak for several seconds.
  Expected: overlay shows Listening and a partial transcript that may change while speaking.
- [ ] Final text inserted after recording stops.
  Steps: continue the previous recording and stop recording.
  Expected: only the final stable result is inserted into the focused app.
- [ ] Preview disabled.
  Steps: disable live preview and dictate.
  Expected: overlay shows Listening without partial text; final insertion still works.
- [ ] Slow preview fallback.
  Steps: use Small model and Accurate preview if available.
  Expected: preview may skip updates but does not block final transcription.
- [ ] `Voxta new line`.
  Steps: dictate `first line Voxta new line second line`.
  Expected: final inserted text contains one line break.
- [ ] `Voxta delete last word`.
  Steps: dictate `Friday Monday Voxta delete last word`.
  Expected: final inserted text keeps `Friday` and removes `Monday`.
- [ ] `Voxta undo`.
  Steps: dictate `Friday Monday Voxta delete last word Voxta undo`.
  Expected: final inserted text includes `Friday Monday`.
- [ ] `Voxta cancel dictation`.
  Steps: dictate a phrase followed by `Voxta cancel dictation`.
  Expected: recording cancels or final processing inserts nothing.
- [ ] Natural command mode.
  Steps: switch to Natural phrases and dictate `first line new line second line`.
  Expected: command is applied without the prefix.
- [ ] Command parser test.
  Steps: enter command phrases in Settings.
  Expected: preview result matches the expected command output.

## AI Cleanup

- [ ] AI cleanup disabled.
  Expected: no cleanup provider is called and dictation works offline after model download.
- [ ] Ollama unavailable fallback.
  Steps: select Ollama while Ollama is not running.
  Expected: original transcription is inserted and a non-blocking cleanup warning is shown.
- [ ] Ollama cleanup when local model is available.
  Steps: start Ollama, select an installed model, enable cleanup, dictate.
  Expected: final text is cleaned locally through Ollama.
- [ ] Remote provider warning.
  Steps: select OpenAI-compatible endpoint.
  Expected: Settings warns that text leaves the device.
- [ ] Remote provider missing key.
  Expected: cleanup fails safely and original transcription is inserted.
- [ ] Cleanup timeout fallback.
  Steps: configure an unreachable endpoint and short timeout.
  Expected: original transcription is inserted after timeout.

## Clipboard

- [ ] Existing text clipboard is restored after insertion.
  Expected: copying text before dictation leaves the same text available afterward.
- [ ] Clipboard fallback inserts into target app.
  Expected: fallback mode inserts reliably and restores clipboard text.
- [ ] Clipboard restoration failure.
  Steps: use a clipboard manager or restricted clipboard scenario if available.
  Expected: Voxta shows a clear clipboard restoration error and returns to Idle.
- [ ] Direct insertion mode works where supported.
  Expected: text appears without permanently changing clipboard text.

## Permissions And Devices

- [ ] Microphone permission denied.
  Expected: Voxta shows a microphone permission/unavailable message and returns to Idle.
- [ ] Accessibility permission denied on macOS.
  Expected: Voxta shows a clear Accessibility permission message before recording proceeds.
- [ ] Microphone disconnection.
  Steps: select an external microphone, disconnect it, then dictate.
  Expected: Voxta shows a microphone unavailable error and returns to Idle.
- [ ] Missing model.
  Steps: select a model that is not downloaded and attempt dictation.
  Expected: Voxta shows a model missing error and no text is inserted.
- [ ] Download interrupted.
  Steps: interrupt network during a model download.
  Expected: partial download is removed or safely ignored, and the UI reports failure.
- [ ] Model checksum invalid.
  Steps: corrupt a downloaded model file and relaunch or list models.
  Expected: the model is not treated as installed.

## Restart And Startup

- [ ] Application restart.
  Expected: shortcut, microphone, model, language, and privacy settings persist.
- [ ] Corrupted configuration recovery.
  Steps: replace the config file with invalid JSON and launch.
  Expected: Voxta backs up the corrupt file, recreates defaults, and shows a clear error path.
- [ ] Login startup.
  Expected: after login, Voxta starts minimized to tray/menu bar and does not show settings unless onboarding is incomplete.

## Privacy

- [ ] No transcript file is created.
  Expected: app config and data directories contain no transcript history.
- [ ] No audio file remains after success.
  Expected: no `.wav`, `.aiff`, `.pcm`, or temporary recording file remains.
- [ ] No audio file remains after cancellation.
  Expected: cancellation leaves no recording file.
- [ ] No unexpected network requests during offline dictation.
  Expected: after model download, dictation works with networking disabled.
