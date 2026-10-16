import { toTimeFailure, useDisconnect } from "@/api/time";
import { Button } from "@/design";
import { FailureNotice } from "./FailureNotice";

/** The connected workspace, with a button to forget the token. */
export function ConnectedView({ workspaceName }: { workspaceName: string }) {
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
