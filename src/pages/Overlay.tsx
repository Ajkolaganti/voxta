import { useEffect, useState } from "react";
import type { AppStatus } from "../types/voxta";

interface OverlayMessage {
  status: AppStatus;
  message?: string;
}

export function Overlay() {
  const [message, setMessage] = useState<OverlayMessage>({
    status: "recording",
    message: "Listening..."
  });

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
    <main className={`overlay overlay-${message.status}`} aria-live="polite">
      <div className="mic-dot" aria-hidden="true">
        <span />
        <span />
        <span />
      </div>
      <div className="overlay-copy">
        <strong>{label}</strong>
      </div>
    </main>
  );
}
