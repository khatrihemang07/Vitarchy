import { useState } from "react";
import type { ActivityTotals } from "@/api/time";
import { Button, OTHER_COLOR, SERIES_COUNT, seriesColor } from "@/design";
import { AppRow } from "./AppRow";
import { formatSpan } from "../shared/duration";

const APPS_SHOWN = 10;

const colorOf = (index: number) => (index < SERIES_COUNT ? seriesColor(index) : OTHER_COLOR);

/** Apps longest first, each opening to its Contexts and titles. Long lists fold after the first ten. */
export function TotalsList({ totals }: { totals: ActivityTotals }) {
  const [showAll, setShowAll] = useState(false);
  const shown = showAll ? totals.apps : totals.apps.slice(0, APPS_SHOWN);
  const longest = totals.apps[0]?.seconds ?? 0;
  return (
    <div className="flex flex-col gap-3">
      <p className="text-sm text-muted-foreground">
        {formatSpan(totals.total_seconds)} in {String(totals.apps.length)} {totals.apps.length === 1 ? "app" : "apps"}
      </p>
      <ul className="flex flex-col">
        {shown.map((app, index) => (
          <AppRow key={app.app} app={app} total={totals.total_seconds} longest={longest} color={colorOf(index)} />
        ))}
      </ul>
      {totals.apps.length > APPS_SHOWN ? (
        <Button variant="ghost" size="sm" className="self-start" onClick={() => { setShowAll(!showAll); }}>
          {showAll ? "Show fewer apps" : `Show all ${String(totals.apps.length)} apps`}
        </Button>
      ) : null}
    </div>
  );
}
