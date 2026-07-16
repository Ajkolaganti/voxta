# Security Policy

## Supported Versions

Security fixes target the latest released minor version.

## Reporting a Vulnerability

Do not open a public issue for a suspected vulnerability.

Email maintainers or use GitHub private vulnerability reporting once the repository is published.

Include:

- Voxta version
- Operating system
- Steps to reproduce
- Impact
- Any relevant logs without private audio or transcript data

## Security Principles

- No cloud transcription.
- No telemetry by default.
- No API keys are required for core dictation.
- Optional remote cleanup API keys must use operating-system credential storage and must not be logged.
- Optional remote cleanup must be user-enabled and must clearly disclose that final transcript text leaves the device.
- No hidden diagnostic uploads.
- Model downloads are checksum-verified.
- Voxta should not run as Administrator or root.
