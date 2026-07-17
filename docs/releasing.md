# Releasing Voxta

Voxta release binaries for general users must be signed before publication.

## Release Rules

- Publish only from `main`.
- Use a semantic version tag such as `v0.2.0`.
- Keep the GitHub Release as a draft until manual QA is complete.
- Do not attach Whisper model binaries.
- Do not attach local logs, recordings, app data, `.env` files, certificates, or build caches.
- Do not describe a release as production-proven until signed macOS and Windows builds have completed manual QA.

## Required GitHub Secrets

macOS signing and notarization:

- `APPLE_CERTIFICATE`: base64-encoded Developer ID Application `.p12`.
- `APPLE_CERTIFICATE_PASSWORD`: password for the `.p12`.
- `APPLE_ID`: Apple ID used for notarization.
- `APPLE_PASSWORD`: app-specific password or notarization credential.
- `APPLE_TEAM_ID`: Apple Developer Team ID.
- `APPLE_SIGNING_IDENTITY`: optional explicit Developer ID Application identity.

Windows signing:

- `WINDOWS_CERTIFICATE`: base64-encoded code-signing `.pfx`.
- `WINDOWS_CERTIFICATE_PASSWORD`: password for the `.pfx`.

The release workflow fails if required signing secrets are missing.

## Creating A Release Candidate

1. Confirm the working tree is clean.
2. Confirm `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` contain the intended version.
3. Run local validation:

```bash
npm ci
npm run typecheck
npm run lint
npm test
npm run build
npm audit --json
cd src-tauri
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cd ..
```

4. Merge the release branch into `main`.
5. Tag the release:

```bash
git checkout main
git pull origin main
git tag v0.2.0
git push origin v0.2.0
```

6. Wait for the `Release` workflow to finish.
7. Review the generated draft GitHub Release.

## Before Publishing The Draft

Download the attached artifacts and verify:

- macOS `.dmg` opens and contains `Voxta.app`.
- macOS `Voxta.app` is signed and notarized.
- Windows `.msi` installs without administrator privileges.
- Windows installer Authenticode signature is valid.
- SHA-256 checksums match the attached `SHA256SUMS-*` files.
- Manual QA checklist passes on macOS and Windows.
- Settings can download a model.
- Dictation works offline after model download.
- No installer artifact contains model binaries, temporary recordings, config files, API keys, or certificates.

## Expected Artifacts

The release workflow uploads platform-specific artifacts named like:

- `Voxta_0.2.0_macos_<tauri-output-name>.dmg`
- `Voxta_0.2.0_windows_<tauri-output-name>.msi`
- `Voxta_0.2.0_windows_<tauri-output-name>.exe`, if the Windows bundler emits an EXE installer
- `SHA256SUMS-macos.txt`
- `SHA256SUMS-windows.txt`

## If A Platform Fails

Do not publish a partial general-user release unless the release notes clearly state the unsupported platform.

If macOS packaging fails with `hdiutil: create failed - Device not configured`, rerun the workflow on GitHub-hosted macOS or a normal macOS signing machine. The local app bundle alone is not a general-user installer.
