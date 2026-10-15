import { useSyncExternalStore } from "react";

const QUERY = "(prefers-color-scheme: dark)";

function subscribe(onChange: () => void) {
  const media = window.matchMedia(QUERY);
  media.addEventListener("change", onChange);
  return () => {
    media.removeEventListener("change", onChange);
  };
}

const getSnapshot = () => window.matchMedia(QUERY).matches;

/** Live value of the OS dark-mode setting. */
export function useSystemPrefersDark(): boolean {
  return useSyncExternalStore(subscribe, getSnapshot);
}
