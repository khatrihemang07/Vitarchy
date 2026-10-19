import { Link } from "@tanstack/react-router";
import { Button, Notice } from "@/design";

/** What the page shows when the Period has no Timeline: a pointer to Import Activity. */
export function EmptyTimeline() {
  return (
    <Notice
      kind="empty"
      title="No Timeline for this Period yet. Import Activity on the Sources tab."
      action={<Button asChild variant="link"><Link to="/time">Go to Sources</Link></Button>}
    >
      The Timeline is built from the Activity of your Sources. Try another Period if you have imported already.
    </Notice>
  );
}
