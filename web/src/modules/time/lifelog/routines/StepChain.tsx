/** Steps in order, joined by arrows, as an ordered list so the order is announced. */
export function StepChain({ labels }: { labels: readonly string[] }) {
  return (
    <ol className="flex flex-wrap items-center gap-x-1.5 gap-y-1 text-sm">
      {labels.map((label, index) => (
        <li key={`${String(index)}-${label}`} className="flex items-center gap-1.5">
          {index > 0 ? <span aria-hidden className="text-muted-foreground">→</span> : null}
          <span className="rounded-md border bg-background/60 px-2 py-0.5">{label}</span>
        </li>
      ))}
    </ol>
  );
}
