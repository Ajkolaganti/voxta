import { useCallback } from "react";
import { useShortcutRecorder } from "../hooks/useShortcutRecorder";

interface Props {
  value: string;
  onChange: (shortcut: string) => void;
}

export function ShortcutField({ value, onChange }: Props) {
  const commit = useCallback((shortcut: string) => onChange(shortcut), [onChange]);
  const { recording, draft, startRecording } = useShortcutRecorder(value, commit);

  return (
    <div className="shortcut-field">
      <output aria-live="polite">{recording ? draft || "Press shortcut..." : value}</output>
      <button type="button" className="secondary" onClick={startRecording}>
        {recording ? "Recording..." : "Change"}
      </button>
    </div>
  );
}
