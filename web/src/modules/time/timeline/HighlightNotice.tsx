import { X } from "lucide-react";
import { Button } from "@/design";

/** Says which app is highlighted and offers to remove it; the figures still cover the whole Period. */
export function HighlightNotice({ app, onClear }: { app: string; onClear: () => void }) {
  return (
    <div role="status" className="flex flex-wrap items-center gap-2 text-sm text-muted-foreground">
      <span>
        Highlighting <strong className="font-medium text-foreground">{app}</strong>. The figures still cover the whole Period.
      </span>
      <Button variant="outline" size="sm" onClick={onClear}>
        <X aria-hidden="true" />
        Remove highlight
      </Button>
    </div>
  );
}
