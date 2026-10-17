import { ActivityImports } from "./ActivityImports";
import { ConnectionCard } from "./ConnectionCard";
import { SyncCard } from "./SyncCard";
import { TimeNav } from "./TimeNav";

/** `/time`: connect Toggl, Sync Time Entries, and Import Activity from Toggl and Clockify. */
export function TimePage() {
  return (
    <div className="flex max-w-3xl flex-col gap-6">
      <header className="animate-fade-up">
        <h1 className="font-display text-4xl font-bold tracking-tight">Time</h1>
        <p className="text-muted-foreground">Your Toggl Time Entries and computer Activity.</p>
      </header>
      <TimeNav />
      <ConnectionCard />
      <SyncCard />
      <ActivityImports />
    </div>
  );
}
