import { useState } from "react";
import type { StatementPreview } from "@/api/finance";
import { OpenedStatement } from "./OpenedStatement";
import { PasswordsCard } from "./PasswordsCard";
import { PendingStatements } from "./PendingStatements";
import { ReaderStatus } from "./ReaderStatus";
import { StatementCard } from "./StatementCard";
import { UploadSection } from "./UploadSection";

/** A Statement being reviewed: just uploaded (with its preview) or opened from the waiting list (loaded by id). */
interface Reviewing {
  id: number;
  preview?: StatementPreview;
}

/** `/finance`: upload Statements, review and confirm them, and keep the Statement passwords. */
export function ImportPage() {
  const [reviewing, setReviewing] = useState<Reviewing[]>([]);
  const add = (items: Reviewing[]) => {
    setReviewing((current) => [...items, ...current.filter((c) => !items.some((i) => i.id === c.id))]);
  };
  return (
    <div className="flex flex-col gap-6">
      <ReaderStatus />
      <UploadSection onUploaded={(previews) => { add(previews.map((preview) => ({ id: preview.id, preview }))); }} />
      {reviewing.map((item) =>
        item.preview ? <StatementCard key={item.id} preview={item.preview} /> : <OpenedStatement key={item.id} id={item.id} />,
      )}
      <PendingStatements excludeIds={new Set(reviewing.map((r) => r.id))} onReview={(id) => { add([{ id }]); }} />
      <PasswordsCard />
    </div>
  );
}
