import { Link } from "@tanstack/react-router";

/** Shown instead of a Report when there is nothing to report on because Toggl was never connected. */
export function NotConnectedNotice() {
  return (
    <div className="rounded-xl border border-dashed border-border p-6 text-center">
      <p className="font-medium">Toggl is not connected yet.</p>
      <p className="mt-1 text-sm text-muted-foreground">
        Reports are built from your Time Entries.{" "}
        <Link to="/time" className="text-primary underline underline-offset-4 focus-ring">
          Connect Toggl and Sync
        </Link>{" "}
        to see them here.
      </p>
    </div>
  );
}
