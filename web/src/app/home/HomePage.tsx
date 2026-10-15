import { formatLongDate, greetingFor } from "../greeting";
import { EmptyModules } from "./EmptyModules";
import { HealthCard } from "./HealthCard";

interface HomePageProps {
  moduleCount: number;
  now?: Date;
}

export function HomePage({ moduleCount, now = new Date() }: HomePageProps) {
  return (
    <div className="flex max-w-3xl flex-col gap-6">
      <header className="animate-fade-up">
        <h1 className="font-mono text-xs tracking-widest text-muted-foreground uppercase">Home</h1>
        <p className="font-display text-4xl font-bold tracking-tight md:text-5xl">
          {greetingFor(now)}
        </p>
        <p className="text-muted-foreground">{formatLongDate(now)}</p>
      </header>
      {moduleCount === 0 ? <EmptyModules /> : null}
      <HealthCard />
    </div>
  );
}
