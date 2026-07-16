# Platform Permissions

## macOS

Voxta needs:

- Microphone permission.
- Accessibility permission.

Microphone permission is needed to record speech.

Accessibility permission is needed for global keyboard monitoring and reliable cross-application text insertion.

Open:

- System Settings
- Privacy & Security
- Microphone
- Accessibility

Then enable Voxta.

If the app was rebuilt locally, macOS may treat the new binary as a different app and require permission again.

## Windows

Voxta needs:

- Microphone permission in Windows privacy settings.
- User-level keyboard/input access.

Voxta should not run as Administrator. If the target application is elevated, Windows may block user-level insertion. In that case Voxta should show an error instead of asking to run elevated.

## Secure Fields

Voxta refuses insertion when platform metadata identifies the focused field as secure or password-like. Some applications do not expose enough metadata for reliable detection.
