import { Check, CircleAlert, LoaderCircle } from "lucide-react";
import type { SaveState } from "@/api/saveStatus";

/** The "Saved" tick, a saving spinner, or the reason a save failed. */
export function SaveStatus({ state }: { state: SaveState }) {
  if (state.kind === "error") {
    return (
      <p role="alert" className="flex items-center gap-1.5 text-sm text-destructive">
        <CircleAlert className="size-4 shrink-0" />
        Not saved. {state.message}
      </p>
    );
  }
  return (
    <p role="status" className="flex h-5 items-center gap-1.5 text-sm text-muted-foreground">
      {state.kind === "saving" ? (
        <>
          <LoaderCircle className="size-4 animate-spin" />
          Saving…
        </>
      ) : null}
      {state.kind === "saved" ? (
        <>
          <Check className="size-4 text-success" />
          Saved
        </>
      ) : null}
    </p>
  );
}
