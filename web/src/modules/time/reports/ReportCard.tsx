import { type ReactNode, useId } from "react";
import { Card } from "@/design";

interface ReportCardProps {
  title: string;
  children: ReactNode;
}

/**
 * A titled Card for a Report section. It does not clip its content (that would cut off
 * focus rings), so the header and the last row round their own corners.
 */
export function ReportCard({ title, children }: ReportCardProps) {
  const titleId = useId();
  return (
    <section aria-labelledby={titleId}>
      <Card className="gap-0 py-0">
        <h2 id={titleId} className="rounded-t-xl border-b border-border px-4 py-3 font-display text-lg font-bold">
          {title}
        </h2>
        {children}
      </Card>
    </section>
  );
}
