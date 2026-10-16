import { toTimeFailure, useTimeStatus } from "@/api/time";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/design";
import { ConnectedView } from "./ConnectedView";
import { ConnectForm } from "./ConnectForm";
import { FailureNotice } from "./FailureNotice";
import { LoadingNotice } from "./LoadingNotice";

/** Connect or disconnect Toggl. The token is typed here, sent once and never shown again. */
export function ConnectionCard() {
  const status = useTimeStatus();
  return (
    <Card>
      <CardHeader>
        <CardTitle>Toggl connection</CardTitle>
        <CardDescription>
          Vitarchy reads your Time Entries from Toggl Track with your API token.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        {status.isPending ? <LoadingNotice>Checking connection…</LoadingNotice> : null}
        {status.isError ? (
          <FailureNotice failure={toTimeFailure(status.error)} />
        ) : null}
        {status.data?.connected ? (
          <ConnectedView workspaceName={status.data.workspace_name ?? "your workspace"} />
        ) : null}
        {status.data && !status.data.connected ? <ConnectForm /> : null}
      </CardContent>
    </Card>
  );
}
