/** Picking each project's colour: its own Toggl colour where that reads well, else the palette. Pure. */
import { seriesColor, usableSeriesColor } from "@/design";

/** A project as the colour choice sees it. */
export interface ColoredProject {
  name: string;
  color?: string | null;
}

/**
 * One colour per project, in the order given (longest first, so the biggest get the palette's best
 * colours). Toggl's own colours are used only when every project has one that shows on both themes
 * and no two are close; otherwise the whole set takes the palette in order (past it, the neutral
 * colour). Mixing the two could paint two projects the same, as a Toggl blue can equal a palette blue.
 */
export function assignProjectColors(projects: readonly ColoredProject[]): Map<string, string> {
  const own: string[] = [];
  for (const project of projects) {
    const color = usableSeriesColor(project.color, own);
    if (!color) return new Map(projects.map((p, index) => [p.name, seriesColor(index)]));
    own.push(color);
  }
  return new Map(projects.map((p, index) => [p.name, own[index] ?? seriesColor(index)]));
}
