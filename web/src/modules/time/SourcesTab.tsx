import { ActivityImportCard } from "./ActivityImportCard";
import { ConnectionCard } from "./ConnectionCard";
import { SyncCard } from "./SyncCard";

/** Where the data comes from: the Toggl connection, Sync and Import. */
export function SourcesTab() {
  return (
    <div className="flex max-w-3xl flex-col gap-6">
      <ConnectionCard />
      <SyncCard />
      <ActivityImportCard />
    </div>
  );
}
