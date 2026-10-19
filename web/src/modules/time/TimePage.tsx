import { ActivityImports } from "./ActivityImports";
import { ConnectionCard } from "./ConnectionCard";
import { SyncCard } from "./SyncCard";
import { TimelineCard } from "./TimelineCard";

/** `/time`: connect Toggl, Sync Time Entries, and Import Activity from Toggl and Clockify and rebuild the Timeline. */
export function TimePage() {
  return (
    <div className="grid gap-6 lg:grid-cols-2">
      <ConnectionCard />
      <SyncCard />
      <ActivityImports />
      <TimelineCard />
    </div>
  );
}
