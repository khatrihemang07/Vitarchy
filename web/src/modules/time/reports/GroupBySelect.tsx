import type { Grouping } from "@/api/time";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/design";

const GROUPINGS: { value: Grouping; label: string }[] = [
  { value: "description", label: "Description" },
  { value: "project", label: "Project" },
  { value: "tag", label: "Tag" },
];
const NONE = "none";

interface GroupBySelectProps {
  group1: Grouping;
  group2: Grouping | undefined;
  onChange: (group1: Grouping, group2: Grouping | undefined) => void;
}

const asGrouping = (value: string): Grouping | undefined => GROUPINGS.find((g) => g.value === value)?.value;

/** Picks the first grouping and an optional second, which can never repeat the first. */
export function GroupBySelect({ group1, group2, onChange }: GroupBySelectProps) {
  return (
    <div className="flex flex-wrap items-center gap-2 text-sm">
      <span className="text-muted-foreground" id="group1-label">Group by</span>
      <Select value={group1} onValueChange={(v) => { onChange(asGrouping(v) ?? group1, group2); }}>
        <SelectTrigger size="sm" aria-labelledby="group1-label">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {GROUPINGS.map((g) => (
            <SelectItem key={g.value} value={g.value}>{g.label}</SelectItem>
          ))}
        </SelectContent>
      </Select>
      <span className="text-muted-foreground" id="group2-label">then</span>
      <Select value={group2 ?? NONE} onValueChange={(v) => { onChange(group1, asGrouping(v)); }}>
        <SelectTrigger size="sm" aria-labelledby="group2-label">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value={NONE}>No second level</SelectItem>
          {GROUPINGS.filter((g) => g.value !== group1).map((g) => (
            <SelectItem key={g.value} value={g.value}>{g.label}</SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
