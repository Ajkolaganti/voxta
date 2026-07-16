import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useMemo, useState } from "react";
import { defaultConfig, voxtaApi } from "../services/voxta";
import type {
  VoxtaConfig,
  MicrophoneDevice,
  ModelDownloadProgress,
  ModelInfo,
  PermissionStatus,
  RuntimeStatus
} from "../types/voxta";

export function useVoxta() {
  const [config, setConfig] = useState<VoxtaConfig>(defaultConfig);
  const [status, setStatus] = useState<RuntimeStatus>({
    status: "idle",
    enabled: true,
    currentModel: defaultConfig.model
  });
  const [microphones, setMicrophones] = useState<MicrophoneDevice[]>([]);
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [permissions, setPermissions] = useState<PermissionStatus>({
    microphone: "unknown",
    accessibility: "unknown",
    inputMonitoring: "unknown"
  });
  const [downloadProgress, setDownloadProgress] = useState<Record<string, ModelDownloadProgress>>({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setError(null);
    const [nextConfig, nextStatus, nextMicrophones, nextModels, nextPermissions] = await Promise.all([
      voxtaApi.getConfig(),
      voxtaApi.getStatus(),
      voxtaApi.listMicrophones(),
      voxtaApi.listModels(),
      voxtaApi.getPermissions()
    ]);
    setConfig(nextConfig);
    setStatus(nextStatus);
    setMicrophones(nextMicrophones);
    setModels(nextModels);
    setPermissions(nextPermissions);
  }, []);

  useEffect(() => {
    refresh()
      .catch((err: unknown) => setError(err instanceof Error ? err.message : String(err)))
      .finally(() => setLoading(false));
  }, [refresh]);

  useEffect(() => {
    const isTauri = Boolean((window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__);
    if (!isTauri) return undefined;

    let unlisten: (() => void) | undefined;
    listen<ModelDownloadProgress>("model-download-progress", (event) => {
      setDownloadProgress((current) => ({
        ...current,
        [event.payload.modelId]: event.payload
      }));
      if (event.payload.done) {
        void refresh();
      }
    })
      .then((nextUnlisten) => {
        unlisten = nextUnlisten;
      })
      .catch((err: unknown) => setError(err instanceof Error ? err.message : String(err)));

    return () => {
      unlisten?.();
    };
  }, [refresh]);

  const saveConfig = useCallback(async (patch: Partial<VoxtaConfig>) => {
    setError(null);
    setConfig((current) => ({ ...current, ...patch }));
    const merged = { ...config, ...patch };
    const saved = await voxtaApi.saveConfig(merged);
    setConfig(saved);
    return saved;
  }, [config]);

  const currentModel = useMemo(
    () => models.find((model) => model.id === config.model) ?? models[0],
    [models, config.model]
  );

  return {
    config,
    status,
    microphones,
    models,
    permissions,
    loading,
    error,
    currentModel,
    downloadProgress,
    refresh,
    saveConfig,
    setError
  };
}
