import { useMemo, useState } from "react";
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
  const [commandTest, setCommandTest] = useState("Voxta new line continue here");
  const [commandResult, setCommandResult] = useState("");
  const [providerMessage, setProviderMessage] = useState("");
  const [apiKey, setApiKey] = useState("");

  const cleanupPrivacy = useMemo(() => {
    if (!store.config.aiCleanup.enabled || store.config.aiCleanup.provider === "none") {
      return {
        cleanup: "Off",
        leavesDevice: "No",
        detail: "Speech recognition: Local"
      };
    }
    if (store.config.aiCleanup.provider === "ollama") {
      return {
        cleanup: "Local through Ollama",
        leavesDevice: "No",
        detail: "Speech recognition: Local"
      };
    }
    return {
      cleanup: "Remote",
      leavesDevice: "Yes",
      detail: "Speech recognition: Local"
    };
  }, [store.config.aiCleanup.enabled, store.config.aiCleanup.provider]);

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
          <div>
            <dt>AI cleanup</dt>
            <dd>{cleanupPrivacy.cleanup}</dd>
          </div>
          <div>
            <dt>Text leaves device</dt>
            <dd>{cleanupPrivacy.leavesDevice}</dd>
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
          <label className="field-label" htmlFor="shortcut-behavior">
            Shortcut behavior
          </label>
          <select
            id="shortcut-behavior"
            value={store.config.shortcutBehavior}
            onChange={(event) =>
              store.saveConfig({
                shortcutBehavior: event.currentTarget.value as typeof store.config.shortcutBehavior
              })
            }
          >
            <option value="hold">Hold to talk</option>
            <option value="toggle">Press once to start, press again to stop</option>
          </select>
          <p className="hint">
            Hold mode is the default. Toggle mode keeps listening after you release the shortcut.
          </p>
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

      <section className="panel">
        <div className="section-heading">
          <div>
            <h2>Smart Dictation</h2>
            <p className="hint">Preview features for live local feedback, local voice commands, and optional cleanup.</p>
          </div>
        </div>

        <div className="smart-grid">
          <div className="smart-block">
            <h3>Live Preview</h3>
            <label className="switch-row">
              <input
                type="checkbox"
                checked={store.config.streaming.enabled}
                onChange={(event) =>
                  store.saveConfig({
                    streaming: { ...store.config.streaming, enabled: event.currentTarget.checked }
                  })
                }
              />
              <span>Enable live transcription preview</span>
            </label>
            <label className="field-label" htmlFor="preview-interval">
              Preview interval
            </label>
            <input
              id="preview-interval"
              type="number"
              min={500}
              max={2000}
              step={50}
              value={store.config.streaming.intervalMs}
              onChange={(event) =>
                store.saveConfig({
                  streaming: {
                    ...store.config.streaming,
                    intervalMs: Number(event.currentTarget.value)
                  }
                })
              }
            />
            <label className="field-label" htmlFor="streaming-quality">
              Streaming quality
            </label>
            <select
              id="streaming-quality"
              value={store.config.streaming.quality}
              onChange={(event) =>
                store.saveConfig({
                  streaming: {
                    ...store.config.streaming,
                    quality: event.currentTarget.value as typeof store.config.streaming.quality
                  }
                })
              }
            >
              <option value="fast">Fast</option>
              <option value="balanced">Balanced</option>
              <option value="accurate">Accurate</option>
            </select>
          </div>

          <div className="smart-block">
            <h3>Voice Commands</h3>
            <label className="switch-row">
              <input
                type="checkbox"
                checked={store.config.voiceCommands.enabled}
                onChange={(event) =>
                  store.saveConfig({
                    voiceCommands: {
                      ...store.config.voiceCommands,
                      enabled: event.currentTarget.checked
                    }
                  })
                }
              />
              <span>Enable voice commands</span>
            </label>
            <label className="field-label" htmlFor="command-mode">
              Command phrase style
            </label>
            <select
              id="command-mode"
              value={store.config.voiceCommands.mode}
              onChange={(event) =>
                store.saveConfig({
                  voiceCommands: {
                    ...store.config.voiceCommands,
                    mode: event.currentTarget.value as typeof store.config.voiceCommands.mode
                  }
                })
              }
            >
              <option value="prefix">Explicit prefix</option>
              <option value="natural">Natural phrases</option>
            </select>
            <label className="field-label" htmlFor="command-prefix">
              Command prefix
            </label>
            <input
              id="command-prefix"
              type="text"
              value={store.config.voiceCommands.prefix}
              onChange={(event) =>
                store.saveConfig({
                  voiceCommands: {
                    ...store.config.voiceCommands,
                    prefix: event.currentTarget.value
                  }
                })
              }
            />
            <details>
              <summary>Supported commands</summary>
              <p className="hint">
                new line, new paragraph, delete last word, delete last sentence, undo, clear dictation,
                cancel dictation.
              </p>
            </details>
            <label className="field-label" htmlFor="command-test">
              Test command parser
            </label>
            <input
              id="command-test"
              type="text"
              value={commandTest}
              onChange={(event) => setCommandTest(event.currentTarget.value)}
            />
            <button
              type="button"
              className="secondary"
              onClick={async () => setCommandResult(await voxtaApi.testVoiceCommands(commandTest))}
            >
              Test parser
            </button>
            {commandResult && <p className="hint">Result: {commandResult}</p>}
          </div>
        </div>

        <details className="advanced-panel">
          <summary>AI Cleanup</summary>
          <div className="smart-grid">
            <div className="smart-block">
              <label className="switch-row">
                <input
                  type="checkbox"
                  checked={store.config.aiCleanup.enabled}
                  onChange={(event) =>
                    store.saveConfig({
                      aiCleanup: {
                        ...store.config.aiCleanup,
                        enabled: event.currentTarget.checked,
                        provider: event.currentTarget.checked
                          ? store.config.aiCleanup.provider
                          : "none"
                      }
                    })
                  }
                />
                <span>Enable AI cleanup</span>
              </label>
              <label className="field-label" htmlFor="cleanup-provider">
                Provider
              </label>
              <select
                id="cleanup-provider"
                value={store.config.aiCleanup.provider}
                onChange={(event) =>
                  store.saveConfig({
                    aiCleanup: {
                      ...store.config.aiCleanup,
                      provider: event.currentTarget.value as typeof store.config.aiCleanup.provider,
                      enabled: event.currentTarget.value !== "none"
                    }
                  })
                }
              >
                <option value="none">None</option>
                <option value="ollama">Ollama</option>
                <option value="openAiCompatible">OpenAI-compatible endpoint</option>
              </select>
              {store.config.aiCleanup.provider === "openAiCompatible" && (
                <p className="banner error">
                  Remote cleanup sends the final transcript to your configured endpoint.
                </p>
              )}
              <label className="field-label" htmlFor="cleanup-endpoint">
                Endpoint
              </label>
              <input
                id="cleanup-endpoint"
                type="url"
                value={store.config.aiCleanup.endpoint}
                onChange={(event) =>
                  store.saveConfig({
                    aiCleanup: {
                      ...store.config.aiCleanup,
                      endpoint: event.currentTarget.value
                    }
                  })
                }
              />
              <label className="field-label" htmlFor="cleanup-model">
                Model
              </label>
              <input
                id="cleanup-model"
                type="text"
                value={store.config.aiCleanup.model}
                onChange={(event) =>
                  store.saveConfig({
                    aiCleanup: {
                      ...store.config.aiCleanup,
                      model: event.currentTarget.value
                    }
                  })
                }
              />
              {store.config.aiCleanup.provider === "openAiCompatible" && (
                <>
                  <label className="field-label" htmlFor="cleanup-key">
                    API key
                  </label>
                  <input
                    id="cleanup-key"
                    type="password"
                    value={apiKey}
                    onChange={(event) => setApiKey(event.currentTarget.value)}
                  />
                  <button
                    type="button"
                    className="secondary"
                    onClick={async () => {
                      const saved = await voxtaApi.saveCleanupApiKey(apiKey);
                      setProviderMessage(saved ? "API key saved securely." : "API key was not saved.");
                      setApiKey("");
                    }}
                  >
                    Save key securely
                  </button>
                </>
              )}
            </div>

            <div className="smart-block">
              <label className="field-label" htmlFor="cleanup-style">
                Cleanup style
              </label>
              <select
                id="cleanup-style"
                value={store.config.aiCleanup.style}
                onChange={(event) =>
                  store.saveConfig({
                    aiCleanup: {
                      ...store.config.aiCleanup,
                      style: event.currentTarget.value as typeof store.config.aiCleanup.style
                    }
                  })
                }
              >
                <option value="light">Light cleanup</option>
                <option value="professional">Professional</option>
                <option value="casual">Casual</option>
                <option value="concise">Concise</option>
                <option value="custom">Custom instructions</option>
              </select>
              <label className="field-label" htmlFor="cleanup-custom">
                Custom instructions
              </label>
              <textarea
                id="cleanup-custom"
                value={store.config.aiCleanup.customInstructions}
                onChange={(event) =>
                  store.saveConfig({
                    aiCleanup: {
                      ...store.config.aiCleanup,
                      customInstructions: event.currentTarget.value
                    }
                  })
                }
              />
              <label className="field-label" htmlFor="cleanup-timeout">
                Timeout seconds
              </label>
              <input
                id="cleanup-timeout"
                type="number"
                min={1}
                max={30}
                value={store.config.aiCleanup.timeoutSeconds}
                onChange={(event) =>
                  store.saveConfig({
                    aiCleanup: {
                      ...store.config.aiCleanup,
                      timeoutSeconds: Number(event.currentTarget.value)
                    }
                  })
                }
              />
              <button
                type="button"
                className="secondary"
                onClick={async () => {
                  const status = await voxtaApi.cleanupProviderStatus();
                  setProviderMessage(`${status.message}${status.models.length ? ` Models: ${status.models.join(", ")}` : ""}`);
                }}
              >
                Test provider
              </button>
              <dl className="privacy-grid compact">
                <div>
                  <dt>Speech recognition</dt>
                  <dd>Local</dd>
                </div>
                <div>
                  <dt>AI cleanup</dt>
                  <dd>{cleanupPrivacy.cleanup}</dd>
                </div>
                <div>
                  <dt>Text leaves device</dt>
                  <dd>{cleanupPrivacy.leavesDevice}</dd>
                </div>
              </dl>
              {providerMessage && <p className="hint">{providerMessage}</p>}
            </div>
          </div>
        </details>
      </section>

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
            <dd>0.2.0</dd>
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
