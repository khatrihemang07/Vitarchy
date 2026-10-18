import { ChevronDown } from "lucide-react";
import { cn } from "@/design";
import { printedInLabel } from "./printedInLabel";

interface PrintingsToggleProps {
  count: number;
  open: boolean;
  /** Id of the region this button shows and hides. */
  controls: string;
  onToggle: () => void;
}

/** The disclosure button of a Transaction printed in several Statements. */
export function PrintingsToggle({ count, open, controls, onToggle }: PrintingsToggleProps) {
  return (
    <button
      type="button"
      aria-expanded={open}
      aria-controls={controls}
      onClick={onToggle}
      className="mt-1 flex w-fit items-center whitespace-nowrap gap-1 rounded-md text-xs text-muted-foreground hover:text-foreground focus-ring"
    >
      <ChevronDown aria-hidden="true" className={cn("size-4 transition-transform", open && "rotate-180")} />
      {printedInLabel(count)}
    </button>
  );
}
