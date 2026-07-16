export type InsertionMode = "auto" | "direct" | "clipboard";
export type AppStatus = "idle" | "recording" | "transcribing" | "inserting" | "paused" | "error";

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
