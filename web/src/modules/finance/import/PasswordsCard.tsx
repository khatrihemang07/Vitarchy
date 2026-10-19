import { usePasswords } from "@/api/finance";
import { Card, CardContent, CardDescription, CardHeader, CardTitle, Notice, QueryNotices, TableSkeleton } from "@/design";
import { errorMessage } from "../failureMessage";
import { PasswordForm } from "./PasswordForm";
import { PASSWORDS_CARD_ID } from "./passwordsAnchor";
import { PasswordRow } from "./PasswordRow";

/** Saved Statement passwords: tried on every locked file, and never shown again once saved. */
export function PasswordsCard() {
  const passwords = usePasswords();
  return (
    <Card id={PASSWORDS_CARD_ID}>
      <CardHeader>
        <CardTitle>Statement passwords</CardTitle>
        <CardDescription>Locked files are opened with these. A password stays on this computer and is never shown again.</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        {!passwords.data ? (
          <QueryNotices inline query={passwords} describeError={errorMessage} skeleton={<TableSkeleton label="Loading passwords" rows={2} columns={2} />} />
        ) : passwords.data.length === 0 ? (
          <Notice inline kind="empty" title="No Statement passwords saved yet." />
        ) : (
          <ul className="flex flex-col gap-2">
            {passwords.data.map((password) => (
              <PasswordRow key={password.id} password={password} />
            ))}
          </ul>
        )}
        <PasswordForm />
      </CardContent>
    </Card>
  );
}
