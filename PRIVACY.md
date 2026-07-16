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

## Network Requests

Voxta may make these network requests:

- User-triggered model downloads from `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-*.bin`.
- User-clicked links to GitHub project pages, documentation, issue templates, or security reporting pages in the user's browser.

Voxta 0.1.0 does not implement automatic updates. If maintainers add updates later, update delivery must be signed and this document must be updated before release.

Voxta does not upload microphone audio, transcripts, configuration files, diagnostics, or model files.

## Clipboard

Clipboard paste mode temporarily sets clipboard text to the dictated text, sends paste, then restores the prior text clipboard content. Rich clipboard formats may not be preserved on every platform in the MVP.

## Diagnostics

Automatic diagnostic upload is not implemented. Diagnostic logging is local process output and must not include recorded audio or transcript text by default.
