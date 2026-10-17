import { useDisconnect } from "@/api/time";
import { MotionButton } from "@/design";
import { errorMessage } from "./failureMessage";
import { TimeNotice } from "./TimeNotice";

/** The connected workspace, with a button to forget the token. */
export function ConnectedView({ workspaceName }: { workspaceName: string }) {
  const disconnect = useDisconnect();
  return (
    <>
      <p className="text-sm">
        Connected to <strong className="font-semibold">{workspaceName}</strong>.
      </p>
      <p className="text-sm text-muted-foreground">
        Disconnecting forgets the token. Time Entries already synced are kept.
      </p>
      {disconnect.isError ? (
        <TimeNotice inline kind="failure" title={errorMessage(disconnect.error)} />
      ) : null}
      <MotionButton
        variant="outline"
        className="mt-auto self-start"
        disabled={disconnect.isPending}
        onClick={() => {
          disconnect.mutate();
        }}
      >
        {disconnect.isPending ? "Disconnecting…" : "Disconnect"}
      </MotionButton>
    </>
  );
}
