import { RadioGroup as RadioGroupPrimitive } from "radix-ui";
import { cn } from "../lib/utils";

export interface SegmentedOption {
  value: string;
  label: string;
}

interface SegmentedControlProps {
  value: string;
  onValueChange: (value: string) => void;
  options: readonly SegmentedOption[];
  /** Accessible name for the group. */
  label: string;
  id?: string;
  className?: string;
}

/** A pill-shaped single choice among a few options (radio semantics). */
export function SegmentedControl({
  value,
  onValueChange,
  options,
  label,
  id,
  className,
}: SegmentedControlProps) {
  return (
    <RadioGroupPrimitive.Root
      id={id}
      aria-label={label}
      value={value}
      onValueChange={onValueChange}
      className={cn("inline-flex max-w-full flex-wrap gap-1 rounded-3xl bg-muted p-1", className)}
    >
      {options.map((option) => (
        <RadioGroupPrimitive.Item
          key={option.value}
          value={option.value}
          className={cn(
            "rounded-full px-3.5 py-1.5 text-sm font-medium whitespace-nowrap text-muted-foreground outline-none",
            "transition-colors duration-(--duration-fast) hover:text-foreground focus-ring",
            "data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground data-[state=checked]:shadow-glow",
          )}
        >
          {option.label}
        </RadioGroupPrimitive.Item>
      ))}
    </RadioGroupPrimitive.Root>
  );
}
