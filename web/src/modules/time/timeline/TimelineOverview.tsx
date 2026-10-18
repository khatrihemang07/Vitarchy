import { Link } from "@tanstack/react-router";
import type { TimelineSummary } from "@/api/time";
import { FadeValue, StatCard } from "@/design";
import { formatCount, formatSpan } from "../format";
import { ChangeBadge } from "../reports/ChangeBadge";
import { type OverviewFigure, overviewFigures, usualNote } from "./overviewFigures";

const FORMATS = { seconds: formatSpan, count: formatCount } as const;

/** The note and link under a tile, when it has either. */
function hintOf({ note, link }: OverviewFigure) {
  if (!note && !link) return undefined;
  return (
    <>
      {note ? <p>{note}</p> : null}
      {link ? (
        <Link to={link.to} className="rounded-sm font-medium text-module hover:underline focus-ring">
          {link.label}
        </Link>
      ) : null}
    </>
  );
}

function Tile({ figure }: { figure: OverviewFigure }) {
  const hint = hintOf(figure);
  return (
    <StatCard
      label={figure.label}
      value={<FadeValue text={FORMATS[figure.unit](figure.value)} className="text-foreground tracking-tight" />}
      {...(hint && { hint })}
      badge={<ChangeBadge delta={figure.change} unit={figure.unit} against="usual" />}
    />
  );
}

/** The Period at a glance: Active, Away, Away unknown, Switches and the longest Focus stretch, each against the Usual. */
export function TimelineOverview({ summary }: { summary: TimelineSummary }) {
  return (
    <section aria-labelledby="timeline-overview-title" className="flex flex-col gap-3">
      <h2 id="timeline-overview-title" className="font-display text-xl font-bold">Overview</h2>
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-5">
        {overviewFigures(summary).map((figure) => (
          <Tile key={figure.label} figure={figure} />
        ))}
      </div>
      <p className="text-xs text-muted-foreground">{usualNote(summary)}</p>
    </section>
  );
}
