import { Button } from "../ui/button";

/** The one Clear filters button of every filter bar and empty state: ghost and small. */
export function ClearFiltersButton({ onClear }: { onClear: () => void }) {
  return (
    <Button variant="ghost" size="sm" onClick={onClear}>
      Clear filters
    </Button>
  );
}
