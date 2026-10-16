import { BarChart3 } from "lucide-react";
import { Card, CardContent } from "@/design";

/** What a Period tab shows while it has no charts: a calm note, not an error. */
export function PeriodEmpty({ message }: { message: string }) {
  return (
    <Card>
      <CardContent className="flex flex-col items-center gap-3 py-10 text-center">
        <BarChart3 aria-hidden className="size-8 text-muted-foreground" />
        <p className="max-w-sm text-sm text-muted-foreground">{message}</p>
      </CardContent>
    </Card>
  );
}
