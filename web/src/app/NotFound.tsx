import { Link, useLocation } from "@tanstack/react-router";
import { useEffect } from "react";
import { Button } from "@/design";
import { createLogger } from "@/lib/log";

const log = createLogger("router");

/** Shown for any unknown URL; logs the path so a mistyped link or stale bookmark is traceable. */
export function NotFound() {
  const { pathname } = useLocation();
  useEffect(() => {
    log.warn("route not found", { path: pathname });
  }, [pathname]);
  return (
    <section className="flex max-w-3xl flex-col items-start gap-3 animate-fade-up">
      <h1 className="font-display text-4xl font-bold tracking-tight md:text-5xl">Page not found</h1>
      <p className="text-muted-foreground">Nothing lives at this address. It may have moved or never existed.</p>
      <Button asChild variant="outline">
        <Link to="/">Back to Home</Link>
      </Button>
    </section>
  );
}
