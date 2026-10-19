import { ChangeBadge, FadeValue, StatCard } from "@/design";
import type { OverviewTile as Tile } from "./overviewTiles";

/** What every tile's change is measured against. */
export const COMPARED_WITH = "the period just before";

/** One Overview figure that fades in when it changes (no counting up), with its change beneath. */
export function OverviewTile({ tile }: { tile: Tile }) {
  return (
    <StatCard
      label={tile.label}
      value={<FadeValue text={tile.value} className="text-foreground tracking-tight" />}
      badge={<ChangeBadge delta={tile.delta} text={tile.change} against={COMPARED_WITH} />}
    />
  );
}
