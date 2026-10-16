import type { ActivityContextTotal } from "@/api/time";
import { BarRow } from "../shared/BarRow";
import { formatShare, formatSpan, fractionOf } from "../shared/duration";

interface TitleRowsProps {
  context: ActivityContextTotal;
  /** The whole Period's seconds, for each row's share. */
  total: number;
  color: string;
  /** Indent level of the rows. */
  level: number;
}

/** The titles inside one Context, longest first, as bar rows. */
export function TitleRows({ context, total, color, level }: TitleRowsProps) {
  const longest = context.titles[0]?.seconds ?? 0;
  return context.titles.map((title) => (
    <BarRow
      key={title.title}
      label={title.title}
      value={formatSpan(title.seconds)}
      share={formatShare(title.seconds, total)}
      fraction={fractionOf(title.seconds, longest)}
      color={color}
      level={level}
    />
  ));
}
