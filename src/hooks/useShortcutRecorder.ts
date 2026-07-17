import { useCallback, useEffect, useState } from "react";

function eventToShortcut(event: KeyboardEvent): string {
  const parts: string[] = [];
  if (event.ctrlKey) parts.push("Ctrl");
  if (event.metaKey) parts.push("Meta");
  if (event.altKey) parts.push("Alt");
  if (event.shiftKey) parts.push("Shift");

  const ignored = new Set(["Control", "Meta", "Alt", "Shift"]);
  if (!ignored.has(event.key)) {
    const key = event.key === " " ? "Space" : event.key.length === 1 ? event.key.toUpperCase() : event.key;
    parts.push(key);
  }

  return parts.join("+");
}

function isCompleteShortcut(shortcut: string): boolean {
  const modifiers = new Set(["Ctrl", "Meta", "Alt", "Shift"]);
  const parts = shortcut.split("+").filter(Boolean);
  const hasModifier = parts.some((part) => modifiers.has(part));
  const trigger = parts.find((part) => !modifiers.has(part));
  if (!trigger) return false;

  return hasModifier || /^F([1-9]|1[0-2])$/.test(trigger);
}

export function useShortcutRecorder(initialValue: string, onCommit: (shortcut: string) => void) {
  const [recording, setRecording] = useState(false);
  const [draft, setDraft] = useState(initialValue);

  useEffect(() => {
    setDraft(initialValue);
  }, [initialValue]);

  useEffect(() => {
    if (!recording) return undefined;

    const onKeyDown = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();
      const shortcut = eventToShortcut(event);
      if (shortcut) {
        setDraft(shortcut);
      }
      if (event.key === "Escape") {
        setRecording(false);
      }
    };

    const onKeyUp = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();
      if (isCompleteShortcut(draft)) {
        onCommit(draft);
        setRecording(false);
      }
    };

    window.addEventListener("keydown", onKeyDown, true);
    window.addEventListener("keyup", onKeyUp, true);
    return () => {
      window.removeEventListener("keydown", onKeyDown, true);
      window.removeEventListener("keyup", onKeyUp, true);
    };
  }, [recording, draft, onCommit]);

  const startRecording = useCallback(() => setRecording(true), []);

  return { recording, draft, startRecording };
}
