import { Overlay } from "./pages/Overlay";
import { Onboarding } from "./pages/Onboarding";
import { Settings } from "./pages/Settings";
import { useVoxta } from "./stores/useVoxta";

function App() {
  const params = new URLSearchParams(window.location.search);
  const mode = params.get("mode");
  const store = useVoxta();

  if (mode === "overlay") {
    return <Overlay />;
  }

  if (store.loading) {
    return (
      <main className="shell loading-shell" aria-busy="true">
        <p>Loading Voxta...</p>
      </main>
    );
  }

  if (!store.config.onboardingComplete) {
    return <Onboarding store={store} />;
  }

  return <Settings store={store} />;
}

export default App;
