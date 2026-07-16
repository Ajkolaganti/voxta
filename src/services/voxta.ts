import { invoke } from "@tauri-apps/api/core";
import type {
  VoxtaConfig,
  MicrophoneDevice,
  ModelInfo,
  PermissionStatus,
  RuntimeStatus,
  CleanupProviderStatus
} from "../types/voxta";

const isTauri = Boolean((window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__);

const defaultConfig: VoxtaConfig = {
  enabled: true,
  shortcut: "Ctrl+Alt+Space",
  microphoneId: "",
  model: "base",
  language: "auto",
  launchAtLogin: false,
  playSounds: true,
  spokenCommands: true,
  trailingSpace: true,
  insertionMode: "auto",
  onboardingComplete: false,
  streaming: {
    enabled: true,
    intervalMs: 750,
    quality: "balanced"
  },
  voiceCommands: {
    enabled: true,
    mode: "prefix",
    prefix: "Voxta"
  },
  aiCleanup: {
    enabled: false,
    provider: "none",
    endpoint: "http://127.0.0.1:11434",
    model: "",
    style: "light",
    customInstructions: "",
    timeoutSeconds: 8
  }
};

const browserPreviewModels: ModelInfo[] = [
  {
    id: "tiny",
    displayName: "Tiny multilingual",
    fileName: "ggml-tiny.bin",
    sizeBytes: 78643200,
    diskSize: "75 MiB",
    languageMode: "multilingual",
    checksumAlgorithm: "sha1",
    checksum: "bd577a113a864445d4c299885e0cb97d4ba92b5f",
    installed: false
  },
  {
    id: "tiny.en",
    displayName: "Tiny English",
    fileName: "ggml-tiny.en.bin",
    sizeBytes: 78643200,
    diskSize: "75 MiB",
    languageMode: "english",
    checksumAlgorithm: "sha1",
    checksum: "c78c86eb1a8faa21b369bcd33207cc90d64ae9df",
    installed: false
  },
  {
    id: "base",
    displayName: "Base multilingual",
    fileName: "ggml-base.bin",
    sizeBytes: 148897792,
    diskSize: "142 MiB",
    languageMode: "multilingual",
    checksumAlgorithm: "sha1",
    checksum: "465707469ff3a37a2b9b8d8f89f2f99de7299dac",
    installed: false
  },
  {
    id: "base.en",
    displayName: "Base English",
    fileName: "ggml-base.en.bin",
    sizeBytes: 148897792,
    diskSize: "142 MiB",
    languageMode: "english",
    checksumAlgorithm: "sha1",
    checksum: "137c40403d78fd54d454da0f9bd998f78703390c",
    installed: false
  },
  {
    id: "small",
    displayName: "Small multilingual",
    fileName: "ggml-small.bin",
    sizeBytes: 488636416,
    diskSize: "466 MiB",
    languageMode: "multilingual",
    checksumAlgorithm: "sha1",
    checksum: "55356645c2b361a969dfd0ef2c5a50d530afd8d5",
    installed: false
  },
  {
    id: "small.en",
    displayName: "Small English",
    fileName: "ggml-small.en.bin",
    sizeBytes: 488636416,
    diskSize: "466 MiB",
    languageMode: "english",
    checksumAlgorithm: "sha1",
    checksum: "db8a495a91d927739e50b3fc1cc4c6b8f6c2d022",
    installed: false
  }
];

async function call<T>(command: string, args?: Record<string, unknown>, fallback?: T): Promise<T> {
  if (!isTauri) {
    if (fallback === undefined) {
      throw new Error(`Command ${command} is unavailable outside Tauri`);
    }
    return fallback;
  }
  return invoke<T>(command, args);
}

export const voxtaApi = {
  getConfig: () => call<VoxtaConfig>("get_config", undefined, defaultConfig),
  saveConfig: (config: VoxtaConfig) => call<VoxtaConfig>("save_config", { config }, config),
  getStatus: () =>
    call<RuntimeStatus>("get_status", undefined, {
      status: "idle",
      enabled: defaultConfig.enabled,
      currentModel: defaultConfig.model
    }),
  listMicrophones: () =>
    call<MicrophoneDevice[]>("list_microphones", undefined, [
      { id: "", name: "System default microphone", isDefault: true }
    ]),
  listModels: () => call<ModelInfo[]>("list_models", undefined, browserPreviewModels),
  downloadModel: (modelId: string) => call<ModelInfo>("download_model", { modelId }),
  deleteModel: (modelId: string) => call<ModelInfo[]>("delete_model", { modelId }, browserPreviewModels),
  getPermissions: () =>
    call<PermissionStatus>("get_permission_status", undefined, {
      microphone: "unknown",
      accessibility: "unknown",
      inputMonitoring: "unknown"
    }),
  openPermissionSettings: (permission: "microphone" | "accessibility" | "inputMonitoring") =>
    call<void>("open_permission_settings", { permission }, undefined),
  testMicrophone: (microphoneId: string) =>
    call<string>("test_microphone", { microphoneId }, "Microphone input detected."),
  testDictation: (text: string) => call<string>("cleanup_test_transcript", { text }, text),
  testVoiceCommands: (text: string) => call<string>("test_voice_command_parser", { text }, text),
  cleanupProviderStatus: () =>
    call<CleanupProviderStatus>("cleanup_provider_status", undefined, {
      available: false,
      message: "Provider checks are available in the desktop app.",
      models: []
    }),
  saveCleanupApiKey: (apiKey: string) => call<boolean>("save_cleanup_api_key", { apiKey }, false),
  hasCleanupApiKey: () => call<boolean>("has_cleanup_api_key", undefined, false)
};

export { defaultConfig };
