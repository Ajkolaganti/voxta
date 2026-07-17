import type { useVoxta } from "../stores/useVoxta";
import { voxtaApi } from "../services/voxta";
import { ShortcutField } from "../components/ShortcutField";
import { PermissionRows } from "../components/PermissionRows";
import { ModelManager } from "../components/ModelManager";

interface Props {
  store: ReturnType<typeof useVoxta>;
}

export function Onboarding({ store }: Props) {
  const complete = async () => {
    await store.saveConfig({ onboardingComplete: true });
  };

  return (
    <main className="shell">
      <header className="titlebar">
        <div>
          <p className="eyebrow">First run</p>
          <h1>Voxta</h1>
        </div>
        <span className="status-pill">Local dictation</span>
      </header>

      {store.error && <p className="banner error">{store.error}</p>}

      <section className="panel">
        <h2>Private by default</h2>
        <p>
          Dictation runs on this computer after the selected Whisper model is downloaded.
          Voxta does not create accounts, analytics events, transcript history, or audio history.
        </p>
      </section>

      <section className="grid two">
        <PermissionRows
          permissions={store.permissions}
          onOpen={(permission) => voxtaApi.openPermissionSettings(permission)}
        />
        <div className="panel">
          <h2>Shortcut</h2>
          <ShortcutField
            value={store.config.shortcut}
            onChange={(shortcut) => store.saveConfig({ shortcut })}
          />
          <label className="field-label" htmlFor="onboarding-shortcut-behavior">
            Shortcut behavior
          </label>
          <select
            id="onboarding-shortcut-behavior"
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
            Default shortcut is F8. Hold mode records while pressed. Toggle mode records until you
            press the shortcut again.
          </p>
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
        <h2>Test dictation</h2>
        <textarea
          className="test-field"
          placeholder="Put the cursor here, use your shortcut, then speak."
          aria-label="Test dictation field"
        />
      </section>

      <footer className="footer-actions">
        <label className="switch-row">
          <input
            type="checkbox"
            checked={store.config.launchAtLogin}
            onChange={(event) => store.saveConfig({ launchAtLogin: event.currentTarget.checked })}
          />
          <span>Launch Voxta when I log in</span>
        </label>
        <button className="primary" type="button" onClick={complete}>
          Finish
        </button>
      </footer>
    </main>
  );
}
