import { Button } from "@/design";

interface EmptyReportProps {
  /** True when filters are narrowing the Report, so clearing them may help. */
  filtered: boolean;
  onClear: () => void;
}

/** Shown when the Period holds no Time Entries that pass the filters. */
export function EmptyReport({ filtered, onClear }: EmptyReportProps) {
  return (
    <div className="flex flex-col items-center gap-3 rounded-xl border border-dashed border-border p-8 text-center">
      <p className="font-medium">{filtered ? "No Time Entries match" : "No Time Entries in this Period"}</p>
      <p className="text-sm text-muted-foreground">
        {filtered ? "Try other filters, or another Period." : "Try another Period, or Sync from the Time page."}
      </p>
      {filtered ? <Button variant="outline" size="sm" onClick={onClear}>Clear filters</Button> : null}
    </div>
  );
}
