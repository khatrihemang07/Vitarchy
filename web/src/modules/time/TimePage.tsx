import { ActivityImports } from "./ActivityImports";
import { ConnectionCard } from "./ConnectionCard";
import { SyncCard } from "./SyncCard";

/** `/time`: connect Toggl, Sync Time Entries, and Import Activity from Toggl and Clockify. */
export function TimePage() {
  return (
    <div className="grid gap-6 lg:grid-cols-2">
      <ConnectionCard />
      <SyncCard />
      <ActivityImports />
    </div>
  );
}
