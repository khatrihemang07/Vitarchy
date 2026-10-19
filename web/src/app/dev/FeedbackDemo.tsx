import { Notice, Progress, TableSkeleton } from "@/design";

/** Notices, a skeleton and a progress bar, with sample text. */
export function FeedbackDemo() {
  return (
    <div className="flex flex-col gap-4">
      <Notice kind="warning" title="A warning-style notice">Explains what to do next.</Notice>
      <Notice kind="failure" title="A failure notice" onRetry={() => undefined} />
      <Notice kind="empty" inline title="An empty state" />
      <TableSkeleton label="Loading sample rows" rows={3} />
      <Progress label="Sample progress" value={60} />
      <Progress label="Sample work of unknown length" />
    </div>
  );
}
