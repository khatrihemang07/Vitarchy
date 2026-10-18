import { useId, type ReactNode } from "react";
import { Card } from "@/design";

interface TimelineSectionProps {
  title: string;
  children: ReactNode;
  /** Something small beside the title, e.g. a clear button. */
  aside?: ReactNode;
  /** The card has no padding (for lists with their own rows). */
  flush?: boolean;
}

/** A titled card, the frame every part of the Timeline page below the overview shares. */
export function TimelineSection({ title, children, aside, flush = false }: TimelineSectionProps) {
  const id = useId();
  return (
    <section aria-labelledby={id} className="flex min-w-0 flex-col gap-3">
      <div className="flex items-center justify-between gap-3">
        <h2 id={id} className="font-display text-xl font-bold">{title}</h2>
        {aside}
      </div>
      <Card className={flush ? "gap-0 overflow-hidden p-0" : "p-4 md:p-5"}>{children}</Card>
    </section>
  );
}
