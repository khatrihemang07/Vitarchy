import { useConfirmedStatements } from "@/api/finance";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/design";
import { FinanceNotice } from "../FinanceNotice";
import { QueryNotices } from "../QueryNotices";
import { ReadAllAgain } from "./ReadAllAgain";
import { StatementRow } from "./StatementRow";

/** The confirmed Statements, each with Read again. */
export function StatementsCard() {
  const statements = useConfirmedStatements();
  return (
    <Card>
      <CardHeader>
        <CardTitle>Statements</CardTitle>
        <CardDescription>Read again runs the current reader over the stored file and shows what would change before anything is saved.</CardDescription>
      </CardHeader>
      <CardContent>
        {!statements.data ? (
          <QueryNotices inline query={statements} loadingTitle="Loading Statements…" />
        ) : statements.data.length === 0 ? (
          <FinanceNotice inline kind="empty" title="No confirmed Statements yet." />
        ) : (
          <div className="flex flex-col gap-4">
            <ReadAllAgain />
            <ul className="flex flex-col gap-3">
              {statements.data.map((statement) => (
                <StatementRow key={statement.id} statement={statement} />
              ))}
            </ul>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
