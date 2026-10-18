import { useAwayPreview } from "@/api/time";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/design";

import { AwayChoiceRow } from "./AwayChoiceRow";
import { awayPreviewRows, hasPreviewData, watchingPlaces } from "./awayPreviewRows";
import { errorMessage } from "./failureMessage";
import { formatSpan } from "./format";
import { TimeNotice } from "./TimeNotice";

/** What Away rules are in force and what other thresholds would have changed; read-only (the rules live in code). */
export function AwayRulesCard() {
  const preview = useAwayPreview();
  const data = preview.data;

  return (
    <Card>
      <CardHeader>
        <CardTitle>Away rules</CardTitle>
        <CardDescription>How the Timeline decides you were away.</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-1 flex-col gap-4">
        {preview.isPending ? <TimeNotice inline kind="loading" title="Loading Away rules…" /> : null}
        {preview.isError ? (
          <TimeNotice inline kind="failure" title={errorMessage(preview.error)} onRetry={() => void preview.refetch()} />
        ) : null}
        {data ? (
          <>
            <p className="text-sm">
              No input for <strong>{formatSpan(data.current_secs)}</strong> or more counts as Away, starting from your last input.
              Shorter pauses stay Active. While watching or in a call ({watchingPlaces(data)}), the first{" "}
              {formatSpan(data.watching_cap_secs)} without input stays Active.
            </p>
            {hasPreviewData(data) ? (
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <caption className="pb-2 text-left text-xs text-muted-foreground">
                    Per Day, over the last {data.days} Days of Clockify Activity, with the change from the current rule.
                  </caption>
                  <thead>
                    <tr className="text-xs text-muted-foreground">
                      <th scope="col" className="px-2 py-1 text-left font-normal">
                        No input for
                      </th>
                      <th scope="col" className="px-2 py-1 text-right font-normal">
                        Away
                      </th>
                      <th scope="col" className="px-2 py-1 text-right font-normal">
                        Active
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    {awayPreviewRows(data).map((row) => (
                      <AwayChoiceRow key={row.key} row={row} />
                    ))}
                  </tbody>
                </table>
              </div>
            ) : (
              <p className="text-sm text-muted-foreground">
                No Clockify Activity in the last {data.days} Days. Import it to see what each threshold would change.
              </p>
            )}
            <ul className="list-disc space-y-1 pl-5 text-sm text-muted-foreground">
              <li>Higher: more pauses count as Active, and short real breaks vanish.</li>
              <li>Lower: stricter, so more of your time counts as Away.</li>
            </ul>
            <p className="text-xs text-muted-foreground">The rules are set in code. Changing them rebuilds the Timeline once.</p>
          </>
        ) : null}
      </CardContent>
    </Card>
  );
}
