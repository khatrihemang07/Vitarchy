import { useTimeStatus } from "@/api/time";
import { QueryNotices } from "@/design";
import { ActivityImportCard } from "./ActivityImportCard";
import { errorMessage } from "./failureMessage";

/** One Import card per Source, from the Time status. */
export function ActivityImports() {
  const status = useTimeStatus();
  if (!status.data) {
    // Spans the grid, so a notice is not squeezed into one card's column.
    return (
      <div className="lg:col-span-2">
        <QueryNotices query={status} describeError={errorMessage} loadingTitle="Loading status…" />
      </div>
    );
  }
  return (
    <>
      {status.data.activity.map((activity) => (
        <ActivityImportCard key={activity.source} activity={activity} />
      ))}
    </>
  );
}
