# Manual QA Checklist

Run this checklist on signed or local release builds before publishing `v0.1.0`.

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
  Steps: open a note, place the cursor in the body, hold the shortcut, say a short sentence, release.
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
