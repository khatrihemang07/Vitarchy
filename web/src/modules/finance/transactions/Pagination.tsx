import { MotionButton } from "@/design";
import { formatCount } from "../money";
import { pageRange } from "../transactionsSearch";

interface PaginationProps {
  total: number;
  page: number;
  onPage: (page: number) => void;
  /** The next page is loading: the old one is still on screen. */
  busy?: boolean;
}

/** Previous and Next with "51 to 100 of 230"; says "Loading…" while the next page loads. */
export function Pagination({ total, page, onPage, busy = false }: PaginationProps) {
  const range = pageRange(total, page);
  return (
    <nav aria-label="Transactions pages" className="flex flex-wrap items-center justify-between gap-3">
      <p role={busy ? "status" : undefined} className="text-sm text-muted-foreground">
        {busy ? "Loading Transactions…" : `${formatCount(range.first)} to ${formatCount(range.last)} of ${formatCount(total)}`}
      </p>
      <div className="flex gap-2">
        <MotionButton variant="outline" size="sm" disabled={busy || !range.hasPrevious} onClick={() => { onPage(page - 1); }}>
          Previous
        </MotionButton>
        <MotionButton variant="outline" size="sm" disabled={busy || !range.hasNext} onClick={() => { onPage(page + 1); }}>
          Next
        </MotionButton>
      </div>
    </nav>
  );
}
