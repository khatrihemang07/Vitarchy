import { formatDate, formatTime } from "../format/datetime";
import { useNow } from "../format/useNow";
import { greeting } from "../greeting";
import { useCoreSettings } from "../settings/useCoreSettings";
import { EmptyModules } from "./EmptyModules";
import { HealthCard } from "./HealthCard";

interface HomePageProps {
  moduleCount: number;
  /** Pins the clock (tests); defaults to the live time. */
  now?: Date;
}

/** Home: greeting, date and time in the person's formats, first-run state and health. */
export function HomePage({ moduleCount, now }: HomePageProps) {
  const live = useNow();
  const date = now ?? live;
  const core = useCoreSettings();
  return (
    <div className="flex flex-col gap-6">
      <header className="animate-fade-up">
        <h1 className="font-mono text-xs tracking-widest text-muted-foreground uppercase">Home</h1>
        <p className="font-display text-4xl font-bold tracking-tight md:text-5xl">
          {greeting(date, core.display_name)}
        </p>
        <p className="text-muted-foreground">
          {formatDate(date, core)} · {formatTime(date, core)}
        </p>
      </header>
      {moduleCount === 0 ? <EmptyModules /> : null}
      <HealthCard />
    </div>
  );
}
