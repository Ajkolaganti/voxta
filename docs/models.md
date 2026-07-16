# Model Downloads

Voxta uses whisper.cpp ggml model files from:

`https://huggingface.co/ggerganov/whisper.cpp`

The app downloads only the model selected by the user.

The production download base URL is:

`https://huggingface.co/ggerganov/whisper.cpp/resolve/main`

## Included Model Choices

| ID | Name | Disk | Mode | Upstream checksum |
| --- | --- | ---: | --- | --- |
| `tiny` | Tiny multilingual | 75 MiB | multilingual | `bd577a113a864445d4c299885e0cb97d4ba92b5f` |
| `tiny.en` | Tiny English | 75 MiB | English-only | `c78c86eb1a8faa21b369bcd33207cc90d64ae9df` |
| `base` | Base multilingual | 142 MiB | multilingual | `465707469ff3a37a2b9b8d8f89f2f99de7299dac` |
| `base.en` | Base English | 142 MiB | English-only | `137c40403d78fd54d454da0f9bd998f78703390c` |
| `small` | Small multilingual | 466 MiB | multilingual | `55356645c2b361a969dfd0ef2c5a50d530afd8d5` |
| `small.en` | Small English | 466 MiB | English-only | `db8a495a91d927739e50b3fc1cc4c6b8f6c2d022` |

The upstream model card labels these values as `SHA`; Voxta stores the algorithm as SHA-1 for these model definitions.

## Download Behavior

- Downloads write to `*.bin.part` first.
- A completed partial file is verified before it is moved into place.
- Failed or interrupted downloads remove the partial file.
- Checksum failure leaves the model uninstalled and surfaces a model error.
- Deleting a model removes the active Voxta copy and any migrated legacy copy where present.
- Voxta recognizes a verified legacy `LocalType` model directory so existing users do not need to download the same model again after the rename.

## Storage Location

Models are stored in the operating system's per-user application data directory:

- macOS: `~/Library/Application Support/Voxta/models`
- Windows: `%APPDATA%\Voxta\models`

## Offline Use

After a model is downloaded and verified, dictation works offline.

## Source And License Notes

The model files are external artifacts from the whisper.cpp model repository and originate from OpenAI Whisper model weights converted for whisper.cpp. Voxta does not redistribute these model binaries in source control. Maintainers should review the upstream model card and applicable model-weight terms before attaching model binaries to any release.
