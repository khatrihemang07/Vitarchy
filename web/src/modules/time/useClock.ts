import { useEffect, useState } from "react";

const TICK_MS = 30_000;

/** The current time, refreshed every 30 seconds so "5 min ago" stays true while the page is open. */
export function useClock(): Date {
  const [now, setNow] = useState(() => new Date());
  useEffect(() => {
    const id = setInterval(() => {
      setNow(new Date());
    }, TICK_MS);
    return () => {
      clearInterval(id);
    };
  }, []);
  return now;
}
