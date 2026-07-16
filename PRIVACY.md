# Privacy

Voxta is designed to keep dictation local.

## What Voxta Stores

By default Voxta stores:

- User settings.
- Downloaded Whisper model files.

By default Voxta does not store:

- Audio recordings.
- Transcript history.
- Account identifiers.
- Analytics events.
- AI cleanup API keys in plain-text configuration.

OpenAI-compatible API keys, when used, are stored with operating-system credential storage where implemented.

## Network Requests

Voxta may make these network requests:

- User-triggered model downloads from `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-*.bin`.
- Optional AI cleanup requests to a local Ollama endpoint, defaulting to `http://127.0.0.1:11434`, only when AI cleanup is enabled and Ollama is selected.
- Optional AI cleanup requests to the user-configured OpenAI-compatible endpoint, only when AI cleanup is enabled and that remote provider is selected.
- User-clicked links to GitHub project pages, documentation, issue templates, or security reporting pages in the user's browser.

Voxta 0.2.0 does not implement automatic updates. If maintainers add updates later, update delivery must be signed and this document must be updated before release.

Voxta does not upload microphone audio, configuration files, diagnostics, or model files. Voxta sends final transcript text only when the user explicitly enables a remote AI cleanup provider.

## Smart Dictation

- Live streaming preview runs through the local Whisper model.
- Voice commands are parsed and applied locally.
- AI cleanup is off by default.
- Ollama cleanup is local to the user's machine when Ollama is bound to localhost.
- OpenAI-compatible cleanup sends the final dictated text to the configured endpoint.
- If cleanup fails, Voxta inserts the original local transcription and shows a non-blocking warning.

## Clipboard

Clipboard paste mode temporarily sets clipboard text to the dictated text, sends paste, then restores the prior text clipboard content. Rich clipboard formats may not be preserved on every platform in the MVP.

## Diagnostics

Automatic diagnostic upload is not implemented. Diagnostic logging is local process output and must not include recorded audio or transcript text by default.
