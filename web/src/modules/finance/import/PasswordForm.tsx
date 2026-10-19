import { type SyntheticEvent, useState } from "react";
import { useAddPassword } from "@/api/finance";
import { Input, Label, MotionButton, Notice } from "@/design";
import { errorMessage } from "../failureMessage";
import { PASSWORD_LABEL_FIELD_ID } from "./passwordsAnchor";

/** Saves a Statement password under a label; the secret is cleared from the field as soon as it is sent. */
export function PasswordForm() {
  const [label, setLabel] = useState("");
  const [secret, setSecret] = useState("");
  const add = useAddPassword();

  const submit = (event: SyntheticEvent) => {
    event.preventDefault();
    add.mutate(
      { label: label.trim(), secret: secret.trim() },
      { onSuccess: () => { setLabel(""); } },
    );
    // Leave the secret out of state as soon as it is sent, whether or not the save works.
    setSecret("");
  };

  return (
    <form onSubmit={submit} className="flex flex-col gap-3">
      <div className="grid gap-3 sm:grid-cols-2">
        <div className="flex flex-col gap-1.5">
          <Label htmlFor={PASSWORD_LABEL_FIELD_ID}>Label</Label>
          <Input
            id={PASSWORD_LABEL_FIELD_ID}
            value={label}
            placeholder="For example, HDFC card"
            onChange={(event) => { setLabel(event.target.value); }}
          />
        </div>
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="statement-password-secret">Password</Label>
          <Input
            id="statement-password-secret"
            type="password"
            autoComplete="new-password"
            spellCheck={false}
            value={secret}
            onChange={(event) => { setSecret(event.target.value); }}
          />
        </div>
      </div>
      {add.isError ? <Notice kind="failure" inline title={errorMessage(add.error)} /> : null}
      <MotionButton type="submit" className="self-start" disabled={add.isPending || label.trim() === "" || secret.trim() === ""}>
        {add.isPending ? "Saving…" : "Save password"}
      </MotionButton>
    </form>
  );
}
