export type InsertionMode = "auto" | "direct" | "clipboard";
export type StreamingQuality = "fast" | "balanced" | "accurate";
export type VoiceCommandMode = "natural" | "prefix";
export type AiCleanupProvider = "none" | "ollama" | "openAiCompatible";
export type CleanupStyle = "light" | "professional" | "casual" | "concise" | "custom";
export type AppStatus =
  | "idle"
  | "recording"
  | "transcribing"
  | "applyingCommands"
  | "cleaningUp"
  | "inserting"
  | "cancelled"
  | "paused"
  | "error";

export interface StreamingConfig {
  enabled: boolean;
  intervalMs: number;
  quality: StreamingQuality;
}

export interface VoiceCommandConfig {
  enabled: boolean;
  mode: VoiceCommandMode;
  prefix: string;
}

export interface AiCleanupConfig {
  enabled: boolean;
  provider: AiCleanupProvider;
  endpoint: string;
  model: string;
  style: CleanupStyle;
  customInstructions: string;
  timeoutSeconds: number;
}

export interface VoxtaConfig {
  enabled: boolean;
  shortcut: string;
  microphoneId: string;
  model: string;
  language: string;
  launchAtLogin: boolean;
  playSounds: boolean;
  spokenCommands: boolean;
  trailingSpace: boolean;
  insertionMode: InsertionMode;
  onboardingComplete: boolean;
  streaming: StreamingConfig;
  voiceCommands: VoiceCommandConfig;
  aiCleanup: AiCleanupConfig;
}

export interface MicrophoneDevice {
  id: string;
  name: string;
  isDefault: boolean;
}

export interface ModelInfo {
  id: string;
  displayName: string;
  fileName: string;
  sizeBytes: number;
  diskSize: string;
  languageMode: "multilingual" | "english";
  checksumAlgorithm: "sha1" | "sha256";
  checksum: string;
  installed: boolean;
  path?: string;
}

export interface PermissionStatus {
  microphone: "granted" | "denied" | "notDetermined" | "unknown";
  accessibility: "granted" | "denied" | "notRequired" | "unknown";
  inputMonitoring: "granted" | "denied" | "notDetermined" | "notRequired" | "unknown";
}

export interface RuntimeStatus {
  status: AppStatus;
  enabled: boolean;
  currentModel: string;
  message?: string;
}

export interface ModelDownloadProgress {
  modelId: string;
  downloadedBytes: number;
  totalBytes?: number;
  done: boolean;
}

export interface CleanupProviderStatus {
  available: boolean;
  message: string;
  models: string[];
}
