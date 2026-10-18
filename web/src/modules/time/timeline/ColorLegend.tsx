import type { AppUsage } from "@/api/time";
import { cn } from "@/design";
import { type AppColors, colorOf, OTHER_COLOR, PALETTE_SIZE } from "./appColors";
import { AWAY_HATCH } from "./markStyle";

interface ColorLegendProps {
  apps: readonly AppUsage[];
  colors: AppColors;
  highlight?: string | undefined;
  onApp: (app: string | undefined) => void;
}

const SWATCH = "inline-block size-3 shrink-0 rounded-sm";

/** What the band's colours mean: the leading apps (click to highlight), Other, Time Entries, Away and Away unknown. */
export function ColorLegend({ apps, colors, highlight, onApp }: ColorLegendProps) {
  const leading = apps.slice(0, PALETTE_SIZE);
  return (
    <ul aria-label="Colours" className="flex flex-wrap gap-x-3 gap-y-1.5 text-xs text-muted-foreground">
      {leading.map(({ app }) => (
        <li key={app}>
          <button
            type="button"
            aria-pressed={highlight === app}
            onClick={() => { onApp(app); }}
            className={cn("flex max-w-40 items-center gap-1.5 rounded-sm px-1 py-0.5 hover:text-foreground focus-ring", highlight === app && "bg-accent text-foreground")}
          >
            <span aria-hidden="true" className={SWATCH} style={{ background: colorOf(colors, app) }} />
            <span className="truncate">{app}</span>
          </button>
        </li>
      ))}
      {apps.length > PALETTE_SIZE ? (
        <li className="flex items-center gap-1.5 px-1 py-0.5"><span aria-hidden="true" className={SWATCH} style={{ background: OTHER_COLOR }} />Other apps</li>
      ) : null}
      <li className="flex items-center gap-1.5 px-1 py-0.5"><span aria-hidden="true" className={`${SWATCH} bg-module/70`} />Time Entries</li>
      <li className="flex items-center gap-1.5 px-1 py-0.5"><span aria-hidden="true" className={SWATCH} style={{ backgroundImage: AWAY_HATCH, boxShadow: "inset 0 0 0 1px var(--away)" }} />Away</li>
      <li className="flex items-center gap-1.5 px-1 py-0.5"><span aria-hidden="true" className={SWATCH} style={{ background: "color-mix(in srgb, var(--foreground) 35%, var(--away-unknown))" }} />Away unknown (faded)</li>
    </ul>
  );
}
