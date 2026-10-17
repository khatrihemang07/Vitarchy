import { toTimeFailure, useTimeStatus } from "@/api/time";

import { ActivityImportCard } from "./ActivityImportCard";
import { FailureNotice } from "./FailureNotice";
import { LoadingNotice } from "./LoadingNotice";

/** One Import card per Source, from the Time status. */
export function ActivityImports() {
  const status = useTimeStatus();
  if (status.isPending) return <LoadingNotice>Loading status…</LoadingNotice>;
  if (status.isError) return <FailureNotice failure={toTimeFailure(status.error)} />;
  return (
    <>
      {status.data.activity.map((activity) => (
        <ActivityImportCard key={activity.source} activity={activity} />
      ))}
    </>
  );
}
