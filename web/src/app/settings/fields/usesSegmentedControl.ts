import type { SelectOption } from "../schema";

/** Up to this many options show as a segmented control; more go in a dropdown. */
const SEGMENTED_MAX_OPTIONS = 4;
/** Labels longer than this in total would not fit side by side. */
const SEGMENTED_MAX_LABEL_CHARS = 40;

/** Few, short options read best as a segmented control; otherwise use a dropdown. */
export function usesSegmentedControl(options: readonly SelectOption[]): boolean {
  const chars = options.reduce((total, option) => total + option.label.length, 0);
  return options.length <= SEGMENTED_MAX_OPTIONS && chars <= SEGMENTED_MAX_LABEL_CHARS;
}
