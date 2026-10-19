import { FadeUp } from "../motion/FadeUp";

/** The big title and one-line subtitle at the top of a Module's pages, entering first in the page's sequence. */
export function ModulePageHeader({ title, subtitle }: { title: string; subtitle: string }) {
  return (
    <FadeUp index={0}>
      <header>
        <h1 className="font-display text-4xl font-bold tracking-tight md:text-5xl">{title}</h1>
        <p className="text-muted-foreground">{subtitle}</p>
      </header>
    </FadeUp>
  );
}
