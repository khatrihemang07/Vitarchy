import type { Overview } from "@/api/finance";
import { Card } from "@/design";
import { MoneyBars } from "./MoneyBars";
import { OverviewTile } from "./OverviewTile";
import { overviewTiles } from "./overviewTiles";

/** Money in, Money out, Net and Transfers at a glance, each against the period just before, over the daily bars. */
export function OverviewStrip({ overview }: { overview: Overview }) {
  return (
    <section aria-labelledby="finance-overview-title" className="flex flex-col gap-3">
      <h2 id="finance-overview-title" className="font-display text-xl font-bold">Overview</h2>
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {overviewTiles(overview).map((tile) => (
          <OverviewTile key={tile.id} tile={tile} />
        ))}
      </div>
      <p className="text-xs text-muted-foreground">Transfers between your Accounts are left out of Money in and Money out.</p>
      {overview.daily.length > 1 ? (
        <Card className="p-4 md:p-5">
          <MoneyBars daily={overview.daily} />
        </Card>
      ) : null}
    </section>
  );
}
