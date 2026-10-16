import { useState } from "react";

const STORAGE_KEY = "vitarchy.time.dismissedRoutineSuggestions";

function read(): Set<string> {
  try {
    const parsed: unknown = JSON.parse(sessionStorage.getItem(STORAGE_KEY) ?? "[]");
    return new Set(Array.isArray(parsed) ? parsed.filter((item): item is string => typeof item === "string") : []);
  } catch {
    return new Set();
  }
}

/** Suggestions hidden for this browser session only (they come back in a new session). Falls back to memory when storage is blocked. */
export function useDismissedSuggestions(): { dismissed: ReadonlySet<string>; dismiss: (key: string) => void } {
  const [dismissed, setDismissed] = useState<ReadonlySet<string>>(read);
  const dismiss = (key: string) => {
    const next = new Set(dismissed).add(key);
    setDismissed(next);
    try {
      sessionStorage.setItem(STORAGE_KEY, JSON.stringify([...next]));
    } catch {
      // Blocked storage: the hiding lasts until this panel unmounts.
    }
  };
  return { dismissed, dismiss };
}
