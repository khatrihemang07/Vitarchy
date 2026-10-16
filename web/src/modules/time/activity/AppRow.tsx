import type { ActivityAppTotal } from "@/api/time";
import { BarRow } from "../shared/BarRow";
import { formatShare, formatSpan, fractionOf } from "../shared/duration";
import { TitleRows } from "./TitleRows";

interface AppRowProps {
  app: ActivityAppTotal;
  /** The whole Period's seconds, for each row's share. */
  total: number;
  /** The longest app's seconds, which a full-length bar stands for. */
  longest: number;
  color: string;
}

/** One app as an expandable row: its Contexts, then their titles. An app with a single Context goes straight to its titles. */
export function AppRow({ app, total, longest, color }: AppRowProps) {
  const [only] = app.contexts;
  const single = app.contexts.length === 1 && only ? only : undefined;
  const longestContext = only?.seconds ?? 0;
  const hasDetail = single ? single.titles.length > 0 : app.contexts.length > 0;
  return (
    <BarRow label={app.app} value={formatSpan(app.seconds)} share={formatShare(app.seconds, total)} fraction={fractionOf(app.seconds, longest)} color={color}>
      {hasDetail
        ? single
          ? <TitleRows context={single} total={total} color={color} level={1} />
          : app.contexts.map((context) => (
              <BarRow
                key={context.name}
                label={context.name}
                value={formatSpan(context.seconds)}
                share={formatShare(context.seconds, total)}
                fraction={fractionOf(context.seconds, longestContext)}
                color={color}
                level={1}
              >
                {context.titles.length > 0 ? <TitleRows context={context} total={total} color={color} level={2} /> : undefined}
              </BarRow>
            ))
        : undefined}
    </BarRow>
  );
}
