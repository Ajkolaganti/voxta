# Building Voxta

## Prerequisites

All platforms:

- Node.js 18+
- npm 10+
- Rust stable
- CMake

macOS:

- Xcode Command Line Tools
- Apple Developer credentials only when signing/notarizing release builds

Windows:

- Visual Studio Build Tools with Desktop C++ workload
- WebView2 runtime

## Development

```bash
npm ci
npm run tauri:dev
```

## Checks

```bash
npm run typecheck
npm run lint
npm test
npm run build
npm audit --json
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

If CMake is not available, run the core Rust tests without the default whisper.cpp feature:

```bash
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets --no-default-features -- -D warnings
```

That build verifies the app shell and core logic but intentionally disables local transcription.

## Release Builds

```bash
npm run tauri:build
```

Expected artifacts:

- macOS: `.dmg`
- Windows: `.msi`

Do not claim artifacts are signed unless real signing credentials were used.

If a local macOS environment cannot create disk images, Tauri may still produce `src-tauri/target/release/bundle/macos/Voxta.app` and then fail during DMG creation with `hdiutil: create failed - Device not configured`. Document the failure and build on a normal macOS runner before publishing.

## Icons

Temporary Early Preview icons are checked in under `src-tauri/icons/`.

To replace them, generate the platform icon set and overwrite:

- `32x32.png`
- `128x128.png`
- `128x128@2x.png`
- `icon.icns`
- `icon.ico`

Keep the small tray/menu-bar shape readable, and keep transparent backgrounds where macOS or Windows tray usage requires them.

## Signing

Repository maintainers can configure signing later through GitHub repository secrets.

Suggested macOS secrets:

- `APPLE_CERTIFICATE`
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_SIGNING_IDENTITY`
- `APPLE_ID`
- `APPLE_PASSWORD`
- `APPLE_TEAM_ID`

Suggested Windows secrets:

- `WINDOWS_CERTIFICATE`
- `WINDOWS_CERTIFICATE_PASSWORD`

Exact signing commands should be validated by maintainers before enabling public signed releases.
