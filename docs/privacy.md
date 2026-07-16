# Privacy Design

Voxta's privacy boundary is simple: microphone audio and transcripts stay on the user's computer.

## No Persistent Audio

Audio is captured into memory. Temporary files are only allowed if a native transcription backend requires them, and must be deleted after transcription, cancellation, or errors.

The current implementation keeps audio in memory for the dictation path.

## No Transcript History

Transcripts are inserted into the focused app and not stored in Voxta history.

## No Telemetry

The repository does not include analytics SDKs, telemetry, cloud logging, or accounts.

## Model Downloads

Model downloads are user-triggered and checksum-verified.

## Clipboard

Clipboard fallback mode temporarily uses the clipboard for reliable insertion, then restores previous text clipboard contents. Native rich clipboard restoration is a known MVP limitation.
