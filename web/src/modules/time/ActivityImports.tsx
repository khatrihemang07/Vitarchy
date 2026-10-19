import { useTimeStatus } from "@/api/time";
import { ActivityImportCard } from "./ActivityImportCard";

/** One Import card per Source, from the Time status. The page shows the status' loading and failure once, so this waits for it. */
export function ActivityImports() {
  const status = useTimeStatus();
  if (!status.data) return null;
  return (
    <>
      {status.data.activity.map((activity) => (
        <ActivityImportCard key={activity.source} activity={activity} />
      ))}
    </>
  );
}
