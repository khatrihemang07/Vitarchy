import { useState } from "react";
import type { ActivityAppTotal, ActivityContextTotal, ActivityTotals } from "@/api/time";
import { Button, OTHER_COLOR, SERIES_COUNT, seriesColor } from "@/design";
import { BarRow } from "./BarRow";
import { formatShare, formatSpan } from "./duration";

const APPS_SHOWN = 10;

const colorOf = (index: number) => (index < SERIES_COUNT ? seriesColor(index) : OTHER_COLOR);

function TitleRows({ context, total, color, level }: { context: ActivityContextTotal; total: number; color: string; level: number }) {
  const longest = context.titles[0]?.seconds ?? 1;
  return context.titles.map((title) => (
    <BarRow key={title.title} label={title.title} value={formatSpan(title.seconds)} share={formatShare(title.seconds, total)} fraction={title.seconds / longest} color={color} level={level} />
  ));
}

function AppRow({ app, total, longest, color }: { app: ActivityAppTotal; total: number; longest: number; color: string }) {
  const [only] = app.contexts;
  // An app with a single Context (most apps) goes straight to its titles.
  const single = app.contexts.length === 1 && only ? only : undefined;
  const longestContext = only?.seconds ?? 1;
  const hasDetail = single ? single.titles.length > 0 : app.contexts.length > 0;
  return (
    <BarRow label={app.app} value={formatSpan(app.seconds)} share={formatShare(app.seconds, total)} fraction={app.seconds / longest} color={color}>
      {hasDetail
        ? single
          ? <TitleRows context={single} total={total} color={color} level={1} />
          : app.contexts.map((context) => (
              <BarRow key={context.name} label={context.name} value={formatSpan(context.seconds)} share={formatShare(context.seconds, total)} fraction={context.seconds / longestContext} color={color} level={1}>
                {context.titles.length > 0 ? <TitleRows context={context} total={total} color={color} level={2} /> : undefined}
              </BarRow>
            ))
        : undefined}
    </BarRow>
  );
}

/** Apps longest first, each opening to its Contexts and titles. Long lists fold after the first ten. */
export function TotalsList({ totals }: { totals: ActivityTotals }) {
  const [showAll, setShowAll] = useState(false);
  const shown = showAll ? totals.apps : totals.apps.slice(0, APPS_SHOWN);
  const longest = totals.apps[0]?.seconds ?? 1;
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
