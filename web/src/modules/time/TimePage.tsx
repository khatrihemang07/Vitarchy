import { ActivityImports } from "./ActivityImports";
import { AwayRulesCard } from "./AwayRulesCard";
import { ConnectionCard } from "./ConnectionCard";
import { SyncCard } from "./SyncCard";
import { TimelineCard } from "./TimelineCard";

/** `/time`: connect Toggl, Sync Time Entries, and Import Activity from Toggl and Clockify, rebuild the Timeline and see the Away rules. */
export function TimePage() {
  return (
    <div className="grid gap-6 lg:grid-cols-2">
      <ConnectionCard />
      <SyncCard />
      <ActivityImports />
      <TimelineCard />
      <AwayRulesCard />
    </div>
  );
}
