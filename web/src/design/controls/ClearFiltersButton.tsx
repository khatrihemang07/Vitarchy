import { MotionButton } from "../motion/MotionButton";
import { Button } from "../ui/button";

interface ClearFiltersButtonProps {
  onClear: () => void;
  /** `bar`: quiet, at the end of a filter bar. `notice`: outlined, as the action of a "nothing matches" notice. */
  placement: "bar" | "notice";
}

/** The one Clear filters button of every filter bar and empty state, in the two looks Time always had. */
export function ClearFiltersButton({ onClear, placement }: ClearFiltersButtonProps) {
  if (placement === "bar") {
    return (
      <Button variant="ghost" size="sm" onClick={onClear}>
        Clear filters
      </Button>
    );
  }
  return (
    <MotionButton variant="outline" onClick={onClear}>
      Clear filters
    </MotionButton>
  );
}
