import type { ModelDownloadProgress, ModelInfo } from "../types/voxta";

interface Props {
  models: ModelInfo[];
  selectedModel: string;
  progress?: Record<string, ModelDownloadProgress>;
  onSelect: (modelId: string) => void;
  onDownload: (modelId: string) => Promise<void>;
  onDelete: (modelId: string) => Promise<void>;
}

export function ModelManager({ models, selectedModel, progress = {}, onSelect, onDownload, onDelete }: Props) {
  return (
    <section className="panel">
      <div className="section-heading">
        <div>
          <h2>Whisper model</h2>
          <p className="hint">Models are stored in the standard per-user application data folder.</p>
        </div>
      </div>
      <div className="model-list" role="list">
        {models.map((model) => (
          <article className="model-row" key={model.id} role="listitem">
            <label>
              <input
                type="radio"
                name="model"
                checked={selectedModel === model.id}
                onChange={() => onSelect(model.id)}
              />
              <span>
                <strong>{model.displayName}</strong>
                <small>
                  {model.diskSize} · {model.languageMode} · {model.checksumAlgorithm.toUpperCase()}
                </small>
              </span>
            </label>
            <div className="row-actions">
              {progress[model.id] && !progress[model.id].done ? (
                <span className="download-progress" aria-live="polite">
                  {formatProgress(progress[model.id])}
                </span>
              ) : model.installed ? (
                <button type="button" className="secondary danger" onClick={() => onDelete(model.id)}>
                  Delete
                </button>
              ) : (
                <button type="button" className="secondary" onClick={() => onDownload(model.id)}>
                  Download
                </button>
              )}
            </div>
          </article>
        ))}
      </div>
    </section>
  );
}

function formatProgress(progress: ModelDownloadProgress) {
  if (progress.totalBytes && progress.totalBytes > 0) {
    return `${Math.round((progress.downloadedBytes / progress.totalBytes) * 100)}%`;
  }
  return `${Math.round(progress.downloadedBytes / 1024 / 1024)} MiB`;
}
