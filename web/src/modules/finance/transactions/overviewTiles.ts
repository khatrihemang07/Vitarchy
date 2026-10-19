import { changeText } from "@/design";
import type { Overview, OverviewFigure } from "@/api/finance";
import { formatInr } from "../money";

/** The four figures of the Overview strip, in order. */
const TILES = [
  { id: "money_in", label: "Money in" },
  { id: "money_out", label: "Money out" },
  { id: "net", label: "Net" },
  { id: "transfers_total", label: "Transfers" },
] as const;

/** One tile: its figure, and how it moved against the range before. */
export interface OverviewTile {
  id: (typeof TILES)[number]["id"];
  label: string;
  /** The figure in INR. */
  value: string;
  /** This range minus the range before, in paise. */
  delta: number;
  /** The change badge's words, e.g. "↑ ₹1,200.00". */
  change: string;
}

const tileOf = (id: OverviewTile["id"], label: string, figure: OverviewFigure): OverviewTile => {
  const delta = figure.paise - figure.previous_paise;
  return { id, label, value: formatInr(figure.paise), delta, change: changeText(delta, (d) => formatInr(Math.abs(d))) };
};

/** The tiles of an Overview. */
export function overviewTiles(overview: Overview): OverviewTile[] {
  return TILES.map(({ id, label }) => tileOf(id, label, overview[id]));
}
