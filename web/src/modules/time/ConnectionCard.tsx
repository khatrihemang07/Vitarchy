import { useTimeStatus } from "@/api/time";
import { Card, CardContent, CardDescription, CardHeader, CardTitle, QueryNotices } from "@/design";
import { ConnectedView } from "./ConnectedView";
import { ConnectForm } from "./ConnectForm";
import { errorMessage } from "./failureMessage";

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
      <CardContent className="flex flex-1 flex-col gap-4">
        <QueryNotices inline query={status} describeError={errorMessage} loadingTitle="Checking connection…" />
        {status.data?.connected ? (
          <ConnectedView workspaceName={status.data.workspace_name ?? "your workspace"} />
        ) : null}
        {status.data && !status.data.connected ? <ConnectForm /> : null}
      </CardContent>
    </Card>
  );
}
