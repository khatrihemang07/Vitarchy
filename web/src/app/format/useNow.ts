import { useEffect, useState } from "react";

const REFRESH_MS = 60_000;

/** The current time, refreshed every minute. */
export function useNow(): Date {
  const [now, setNow] = useState(() => new Date());
  useEffect(() => {
    const id = setInterval(() => {
      setNow(new Date());
    }, REFRESH_MS);
    return () => {
      clearInterval(id);
    };
  }, []);
  return now;
}
