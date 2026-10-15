import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../lib/utils";

const badgeVariants = cva(
  "grid h-9 w-8 cursor-default place-items-center font-display text-sm font-extrabold text-on-accent clip-hex",
  {
    variants: {
      tone: {
        gold: "bg-gold",
        mint: "bg-mint",
        sky: "bg-sky",
        locked: "bg-muted text-muted-foreground",
      },
    },
    defaultVariants: { tone: "gold" },
  },
);

interface AchievementBadgeProps extends VariantProps<typeof badgeVariants> {
  /** One or two characters shown on the badge. */
  glyph: string;
  /** What the achievement is, e.g. "Early bird". */
  label: string;
  className?: string;
}

export function AchievementBadge({ glyph, label, tone, className }: AchievementBadgeProps) {
  return (
    <span role="img" aria-label={label} title={label} className={cn(badgeVariants({ tone }), className)}>
      <span aria-hidden="true">{glyph}</span>
    </span>
  );
}
