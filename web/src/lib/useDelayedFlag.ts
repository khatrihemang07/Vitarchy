import { useEffect, useState } from "react";

/**
 * Follows `active`, but only turns true once it has stayed true for `delayMs`; it turns false
 * at once. Used so a loading state that ends quickly never flashes on screen. With no delay it
 * is simply `active`.
 */
export function useDelayedFlag(active: boolean, delayMs: number): boolean {
  const [elapsed, setElapsed] = useState(false);
  useEffect(() => {
    if (!active || delayMs <= 0) return;
    const timer = setTimeout(() => {
      setElapsed(true);
    }, delayMs);
    return () => {
      clearTimeout(timer);
      setElapsed(false);
    };
  }, [active, delayMs]);
  return active && (delayMs <= 0 || elapsed);
}
