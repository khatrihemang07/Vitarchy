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

function write(keys: ReadonlySet<string>) {
  try {
    sessionStorage.setItem(STORAGE_KEY, JSON.stringify([...keys]));
  } catch {
    // Blocked storage: the hiding lasts until this panel unmounts.
  }
}

/** Suggestions hidden for this browser session only (they come back in a new session). Falls back to memory when storage is blocked. */
export function useDismissedSuggestions(): { dismissed: ReadonlySet<string>; dismiss: (key: string) => void } {
  const [dismissed, setDismissed] = useState<ReadonlySet<string>>(read);
  const dismiss = (key: string) => {
    setDismissed((current) => {
      const next = new Set(current).add(key);
      write(next);
      return next;
    });
  };
  return { dismissed, dismiss };
}
