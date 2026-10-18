import type { AccountSummary } from "@/api/finance";
import { Badge, Card, CardContent, CardHeader } from "@/design";
import { maskOrNone } from "../mask";
import { accountKindLabel } from "../statementCopy";
import { accountFigures } from "./accountFigures";
import { RenameField } from "./RenameField";

/** One Account: its name (renamed in place), issuer, kind, masked number and figures. */
export function AccountCard({ account }: { account: AccountSummary }) {
  return (
    <Card data-account-id={account.id}>
      <CardHeader className="gap-2">
        <RenameField account={account} />
        <div className="flex flex-wrap items-center gap-2 text-sm text-muted-foreground">
          <span>{account.issuer}</span>
          <Badge variant="secondary">{accountKindLabel(account.kind)}</Badge>
        </div>
        <p className="text-sm text-muted-foreground tabular-nums">{maskOrNone(account.mask)}</p>
      </CardHeader>
      <CardContent>
        <dl className="grid grid-cols-2 gap-x-4 gap-y-3 text-sm">
          {accountFigures(account).map((figure) => (
            <div key={figure.label} className="flex flex-col">
              <dt className="text-muted-foreground">{figure.label}</dt>
              <dd className="font-medium tabular-nums">{figure.value}</dd>
            </div>
          ))}
        </dl>
      </CardContent>
    </Card>
  );
}
