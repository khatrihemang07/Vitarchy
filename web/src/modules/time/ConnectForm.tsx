import { type SyntheticEvent, useId, useState } from "react";
import { useSetConnection } from "@/api/time";
import { Input, Label, MotionButton, Notice } from "@/design";
import { errorMessage } from "./failureMessage";

/** Asks for the Toggl API token and sends it once; it is cleared from the field as soon as it is sent. */
export function ConnectForm() {
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
    <form onSubmit={submit} className="flex flex-1 flex-col gap-3">
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
      {connect.isError ? (
        <Notice inline kind="failure" title={errorMessage(connect.error)} />
      ) : null}
      <MotionButton type="submit" className="mt-auto self-start" disabled={connect.isPending || token.trim() === ""}>
        {connect.isPending ? "Connecting…" : "Connect"}
      </MotionButton>
    </form>
  );
}
