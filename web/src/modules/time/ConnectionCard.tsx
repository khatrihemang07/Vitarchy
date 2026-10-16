import { type SyntheticEvent, useId, useState } from "react";
import { toTimeFailure, useDisconnect, useSetConnection, useTimeStatus } from "@/api/time";
import { Button, Card, CardContent, CardDescription, CardHeader, CardTitle, Input, Label } from "@/design";
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

function ConnectedView({ workspaceName }: { workspaceName: string }) {
  const disconnect = useDisconnect();
  return (
    <>
      <p>
        Connected to <strong className="font-semibold">{workspaceName}</strong>.
      </p>
      <p className="text-sm text-muted-foreground">
        Disconnecting forgets the token. Time Entries already synced are kept.
      </p>
      {disconnect.isError ? <FailureNotice failure={toTimeFailure(disconnect.error)} /> : null}
      <Button
        variant="outline"
        className="self-start"
        disabled={disconnect.isPending}
        onClick={() => {
          disconnect.mutate();
        }}
      >
        {disconnect.isPending ? "Disconnecting…" : "Disconnect"}
      </Button>
    </>
  );
}

function ConnectForm() {
  const inputId = useId();
  const [token, setToken] = useState("");
  const connect = useSetConnection();

  const submit = (event: SyntheticEvent) => {
    event.preventDefault();
    // Leave the token out of state as soon as it is sent.
    connect.mutate(token.trim(), {
      onSuccess: () => {
        setToken("");
      },
    });
  };

  return (
    <form onSubmit={submit} className="flex flex-col gap-3">
      <div className="flex flex-col gap-2">
        <Label htmlFor={inputId}>Toggl API token</Label>
        <Input
          id={inputId}
          type="password"
          autoComplete="off"
          spellCheck={false}
          value={token}
          onChange={(event) => {
            setToken(event.target.value);
          }}
          aria-invalid={connect.isError}
        />
        <p className="text-sm text-muted-foreground">
          Find it at the bottom of your Toggl profile page. It stays on this computer.
        </p>
      </div>
      {connect.isError ? <FailureNotice failure={toTimeFailure(connect.error)} /> : null}
      <Button type="submit" className="self-start" disabled={connect.isPending || token.trim() === ""}>
        {connect.isPending ? "Connecting…" : "Connect"}
      </Button>
    </form>
  );
}
