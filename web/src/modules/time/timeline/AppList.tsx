import { useState } from "react";
import type { AppUsage } from "@/api/time";
import { type AppColors, colorOf } from "./appColors";
import { AppRow } from "./AppRow";

/** How many apps show before "Show all"; the long tail adds little. */
export const APP_LIMIT = 10;

interface AppListProps {
  apps: readonly AppUsage[];
  colors: AppColors;
  highlight?: string | undefined;
  /** Called with the app to highlight, or the highlighted one again to clear it. */
  onApp: (app: string | undefined) => void;
}

/** Apps ranked by Active time. A row opens to its titles and sites; its name highlights it everywhere. */
export function AppList({ apps, colors, highlight, onApp }: AppListProps) {
  const [openApp, setOpenApp] = useState<string>();
  const [showAll, setShowAll] = useState(false);
  const shown = showAll ? apps : apps.slice(0, APP_LIMIT);
  if (apps.length === 0) return <p className="px-4 py-3 text-sm text-muted-foreground">No Active time in this Period.</p>;
  return (
    <div>
      <ul className="divide-y divide-border">
        {shown.map((usage) => (
          <AppRow
            key={usage.app}
            usage={usage}
            color={colorOf(colors, usage.app)}
            open={openApp === usage.app}
            highlighted={highlight === usage.app}
            onToggleOpen={() => { setOpenApp(openApp === usage.app ? undefined : usage.app); }}
            onHighlight={() => { onApp(usage.app); }}
          />
        ))}
      </ul>
      {shown.length < apps.length ? (
        <button type="button" onClick={() => { setShowAll(true); }} className="w-full rounded-b-xl px-4 py-2 text-left text-sm font-medium text-module hover:underline focus-ring-inset">
          Show all {apps.length} apps
        </button>
      ) : null}
    </div>
  );
}
