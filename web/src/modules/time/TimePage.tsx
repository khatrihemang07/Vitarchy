import { useTimeStatus } from "@/api/time";
import { CardSkeleton, FadeUp, QueryNotices } from "@/design";
import { ActivityImports } from "./ActivityImports";
import { ConnectionCard } from "./ConnectionCard";
import { errorMessage } from "./failureMessage";
import { SyncCard } from "./SyncCard";
import { TimelineCard } from "./TimelineCard";

const GRID = "grid gap-6 lg:grid-cols-2";

/**
 * `/time`: connect Toggl, Sync Time Entries, and Import Activity from Toggl and Clockify and rebuild
 * the Timeline. Loading and failure of the status show once for the whole page, not once per card.
 */
export function TimePage() {
  const status = useTimeStatus();
  if (!status.data) {
    const skeleton = (
      <div className={GRID}>
        <CardSkeleton label="Loading status" />
        <CardSkeleton label="Loading status" />
      </div>
    );
    return <QueryNotices query={status} describeError={errorMessage} skeleton={skeleton} />;
  }
  return (
    <FadeUp index={2}>
      <div className={GRID}>
        <ConnectionCard />
        <SyncCard />
        <ActivityImports />
        <TimelineCard />
      </div>
    </FadeUp>
  );
}
