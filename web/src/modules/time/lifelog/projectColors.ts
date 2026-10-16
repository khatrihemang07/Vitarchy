/** Picking each project's colour: its own Toggl colour where that reads well, else the palette. Pure. */
import { isDistinct, OTHER_COLOR, SERIES_COUNT, SERIES_HEX, seriesColor, usableSeriesColor } from "@/design";

/** What the server calls entries without a project (kept equal to the server's by `scripts/test-guards.sh`). */
export const NO_PROJECT = "No project";

/** A project as the colour choice sees it. */
export interface ColoredProject {
  name: string;
  color?: string | null;
}

/** The first palette slot from `from` on that looks like none of the `own` colours,. */
function freePaletteSlot(from: number, own: readonly string[]): number {
  for (let index = from; index < SERIES_COUNT; index += 1) {
    const hex = SERIES_HEX[index];
    if (hex && isDistinct(hex.light, own) && isDistinct(hex.dark, own)) return index;
  }
  return SERIES_COUNT;
}

/**
 * One colour per project, in the order given (longest first, so the biggest get the palette's best
 * colours). A project keeps its own Toggl colour when it shows on both themes and is not close to
 * one already kept; each other project takes the next palette colour that looks like none of the
 * kept ones, and past the palette the neutral colour. "No project" is always neutral and never
 * takes part, since it has no colour of its own to lose.
 */
export function assignProjectColors(projects: readonly ColoredProject[]): Map<string, string> {
  const colors = new Map<string, string>();
  const own: string[] = [];
  const unmatched: string[] = [];
  for (const project of projects) {
    if (project.name === NO_PROJECT) {
      colors.set(project.name, OTHER_COLOR);
      continue;
    }
    const color = usableSeriesColor(project.color, own);
    if (color) {
      own.push(color);
      colors.set(project.name, color);
    } else {
      unmatched.push(project.name);
    }
  }
  let next = 0;
  for (const name of unmatched) {
    next = freePaletteSlot(next, own);
    colors.set(name, seriesColor(next));
    next += 1;
  }
  return new Map(projects.map((project) => [project.name, colors.get(project.name) ?? OTHER_COLOR]));
}
