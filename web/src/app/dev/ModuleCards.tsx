import { Clock, Wallet } from "lucide-react";
import { accentScope, BorderBeam, MagicCard, NumberTicker, StatCard } from "@/design";

const timeScope = accentScope("var(--module-time)");
const financeScope = accentScope("var(--module-finance)");

/** Each wrapper scopes --module-accent, exactly as the shell does per route. */
export function ModuleCards() {
  return (
    <div className="grid gap-4 sm:grid-cols-2">
      <div style={timeScope}>
        <StatCard
          icon={Clock}
          label="Focus today"
          value={<NumberTicker value={6.5} decimalPlaces={1} className="text-foreground" />}
          hint="hours, 82% of daily goal"
        />
      </div>
      <div style={financeScope}>
        <StatCard
          icon={Wallet}
          label="Budget used"
          value={<NumberTicker value={69} className="text-foreground" />}
          hint="percent of this month"
        />
      </div>
      <div style={timeScope}>
        <MagicCard
          className="rounded-xl p-5"
          gradientColor="var(--accent)"
          gradientFrom="var(--module-time)"
          gradientTo="var(--brand)"
        >
          <p className="font-display font-semibold">MagicCard</p>
          <p className="text-sm text-muted-foreground">Move the pointer: a spotlight follows it.</p>
        </MagicCard>
      </div>
      <div style={financeScope} className="relative overflow-hidden rounded-xl border border-border bg-card p-5">
        <p className="font-display font-semibold">BorderBeam</p>
        <p className="text-sm text-muted-foreground">A light that travels the card edge.</p>
        <BorderBeam size={120} duration={6} colorFrom="var(--module-finance)" colorTo="var(--brand)" />
      </div>
    </div>
  );
}
