import { useEffect, useState } from "react";
import type { AppStatus } from "../types/voxta";

interface OverlayMessage {
  status: AppStatus;
  message?: string;
  preview?: string;
}

export function Overlay() {
  const [message, setMessage] = useState<OverlayMessage>({
    status: "recording",
    message: "Listening..."
  });

  useEffect(() => {
    document.documentElement.classList.add("overlay-document");
    document.body.classList.add("overlay-body");
    return () => {
      document.documentElement.classList.remove("overlay-document");
      document.body.classList.remove("overlay-body");
    };
  }, []);

  useEffect(() => {
    const handler = (event: Event) => {
      const custom = event as CustomEvent<OverlayMessage>;
      setMessage(custom.detail);
    };
    window.addEventListener("voxta-overlay", handler);
    return () => window.removeEventListener("voxta-overlay", handler);
  }, []);

  const label =
    message.message ??
    (message.status === "transcribing"
      ? "Transcribing..."
      : message.status === "error"
        ? "Dictation unavailable"
        : "Listening...");

  return (
    <main className="overlay-shell" aria-live="polite">
      <section className={`overlay-card overlay-${message.status}`} role="status">
        <div className="mic-dot" aria-hidden="true">
          <span />
          <span />
          <span />
        </div>
        <div className="overlay-copy">
          <strong>{label}</strong>
          {message.preview && <span>{message.preview}</span>}
        </div>
      </section>
    </main>
  );
}
