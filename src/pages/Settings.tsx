import type { useVoxta } from "../stores/useVoxta";
import { voxtaApi } from "../services/voxta";
import { ShortcutField } from "../components/ShortcutField";
import { ModelManager } from "../components/ModelManager";
import { PermissionRows } from "../components/PermissionRows";

interface Props {
  store: ReturnType<typeof useVoxta>;
}

const languages = [
  ["auto", "Detect automatically"],
  ["en", "English"],
  ["es", "Spanish"],
  ["fr", "French"],
  ["de", "German"],
  ["it", "Italian"],
  ["pt", "Portuguese"],
  ["ja", "Japanese"],
  ["ko", "Korean"],
  ["zh", "Chinese"]
];

export function Settings({ store }: Props) {
  return (
    <main className="shell">
      <header className="titlebar">
        <div>
          <p className="eyebrow">Settings</p>
          <h1>Voxta</h1>
        </div>
        <span className={`status-pill status-${store.status.status}`}>
          {store.status.enabled ? store.status.status : "paused"}
        </span>
      </header>

      {store.error && <p className="banner error">{store.error}</p>}
      {store.status.message && <p className="banner info">{store.status.message}</p>}

      <section className="toolbar panel">
        <div>
          <h2>Voxta — Private voice typing that works everywhere.</h2>
          <p className="hint">Current model: {store.currentModel?.displayName ?? "None selected"}</p>
        </div>
        <button
          type="button"
          className={store.config.enabled ? "secondary" : "primary"}
          onClick={() => store.saveConfig({ enabled: !store.config.enabled })}
        >
          {store.config.enabled ? "Pause Voxta" : "Start Voxta"}
        </button>
      </section>

      <section className="panel">
        <h2>Privacy status</h2>
        <dl className="privacy-grid">
          <div>
            <dt>Transcription</dt>
            <dd>Transcription runs locally</dd>
          </div>
          <div>
            <dt>Current downloaded model</dt>
            <dd>
              {store.currentModel?.installed
                ? store.currentModel.displayName
                : "No selected model downloaded"}
            </dd>
          </div>
          <div>
            <dt>Internet required</dt>
            <dd>Only for downloading models; updates are not implemented</dd>
          </div>
          <div>
            <dt>Audio storage</dt>
            <dd>Never</dd>
          </div>
          <div>
            <dt>Transcript history</dt>
            <dd>Disabled</dd>
          </div>
          <div>
            <dt>Cloud transcription</dt>
            <dd>Never</dd>
          </div>
        </dl>
        <p className="hint">Automatic updates are not implemented in this early preview.</p>
      </section>

      <section className="grid two">
        <div className="panel">
          <h2>Shortcut</h2>
          <ShortcutField
            value={store.config.shortcut}
            onChange={(shortcut) => store.saveConfig({ shortcut })}
          />
        </div>

        <div className="panel">
          <h2>Microphone</h2>
          <label className="field-label" htmlFor="microphone">
            Input device
          </label>
          <select
            id="microphone"
            value={store.config.microphoneId}
            onChange={(event) => store.saveConfig({ microphoneId: event.currentTarget.value })}
          >
            {store.microphones.map((device) => (
              <option key={device.id || "default"} value={device.id}>
                {device.name}
              </option>
            ))}
          </select>
          <button
            type="button"
            className="secondary"
            onClick={async () => {
              const message = await voxtaApi.testMicrophone(store.config.microphoneId);
              store.setError(message);
            }}
          >
            Test microphone
          </button>
        </div>
      </section>

      <ModelManager
        models={store.models}
        selectedModel={store.config.model}
        progress={store.downloadProgress}
        onSelect={(model) => store.saveConfig({ model })}
        onDownload={async (model) => {
          await voxtaApi.downloadModel(model);
          await store.refresh();
        }}
        onDelete={async (model) => {
          await voxtaApi.deleteModel(model);
          await store.refresh();
        }}
      />

      <section className="grid two">
        <div className="panel">
          <h2>Language</h2>
          <label className="field-label" htmlFor="language">
            Recognition language
          </label>
          <select
            id="language"
            value={store.config.language}
            onChange={(event) => store.saveConfig({ language: event.currentTarget.value })}
          >
            {languages.map(([value, label]) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
          </select>
        </div>

        <div className="panel">
          <h2>Insertion</h2>
          <label className="field-label" htmlFor="insertion-mode">
            Text insertion mode
          </label>
          <select
            id="insertion-mode"
            value={store.config.insertionMode}
            onChange={(event) =>
              store.saveConfig({ insertionMode: event.currentTarget.value as typeof store.config.insertionMode })
            }
          >
            <option value="auto">Automatic reliability mode</option>
            <option value="direct">Direct keyboard insertion</option>
            <option value="clipboard">Clipboard paste fallback</option>
          </select>
          <label className="switch-row">
            <input
              type="checkbox"
              checked={store.config.trailingSpace}
              onChange={(event) => store.saveConfig({ trailingSpace: event.currentTarget.checked })}
            />
            <span>Add a trailing space after dictation</span>
          </label>
        </div>
      </section>

      <section className="grid two">
        <div className="panel">
          <h2>Behavior</h2>
          <label className="switch-row">
            <input
              type="checkbox"
              checked={store.config.launchAtLogin}
              onChange={(event) => store.saveConfig({ launchAtLogin: event.currentTarget.checked })}
            />
            <span>Launch at login</span>
          </label>
          <label className="switch-row">
            <input
              type="checkbox"
              checked={store.config.playSounds}
              onChange={(event) => store.saveConfig({ playSounds: event.currentTarget.checked })}
            />
            <span>Play start and stop sounds</span>
          </label>
          <label className="switch-row">
            <input
              type="checkbox"
              checked={store.config.spokenCommands}
              onChange={(event) => store.saveConfig({ spokenCommands: event.currentTarget.checked })}
            />
            <span>Spoken new-line commands</span>
          </label>
        </div>

        <PermissionRows
          permissions={store.permissions}
          onOpen={(permission) => voxtaApi.openPermissionSettings(permission)}
        />
      </section>

      <section className="panel">
        <h2>Test dictation</h2>
        <textarea
          className="test-field"
          placeholder="Use your shortcut here to test local insertion."
          aria-label="Test dictation field"
        />
      </section>

      <section className="panel about">
        <h2>About</h2>
        <dl>
          <div>
            <dt>Version</dt>
            <dd>0.1.0</dd>
          </div>
          <div>
            <dt>Repository</dt>
            <dd>
              <a href="https://github.com/voxta/voxta" target="_blank" rel="noreferrer">
                github.com/voxta/voxta
              </a>
            </dd>
          </div>
        </dl>
      </section>
    </main>
  );
}
