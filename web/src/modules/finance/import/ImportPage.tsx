import { useState } from "react";
import type { StatementPreview } from "@/api/finance";
import { FadeUp } from "@/design";
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

/** Header and tabs are 0 and 1 of the page's entrance sequence; the sections follow. */
const FIRST_STAGGER = 2;
/** Statements under review follow the upload (3), then the waiting list and the passwords, in the order shown. */
const REVIEWING_STAGGER = FIRST_STAGGER + 2;

/** `/finance`: upload Statements, review and confirm them, and keep the Statement passwords. */
export function ImportPage() {
  const [reviewing, setReviewing] = useState<Reviewing[]>([]);
  const add = (items: Reviewing[]) => {
    setReviewing((current) => [...items, ...current.filter((c) => !items.some((i) => i.id === c.id))]);
  };
  return (
    <div className="flex flex-col gap-6">
      <ReaderStatus stagger={FIRST_STAGGER} />
      <FadeUp index={FIRST_STAGGER + 1}>
        <UploadSection onUploaded={(previews) => { add(previews.map((preview) => ({ id: preview.id, preview }))); }} />
      </FadeUp>
      {reviewing.map((item, position) => (
        <FadeUp key={item.id} index={REVIEWING_STAGGER + position}>
          {item.preview ? <StatementCard preview={item.preview} /> : <OpenedStatement id={item.id} />}
        </FadeUp>
      ))}
      <FadeUp index={REVIEWING_STAGGER + reviewing.length}>
        <PendingStatements excludeIds={new Set(reviewing.map((r) => r.id))} onReview={(id) => { add([{ id }]); }} />
      </FadeUp>
      <FadeUp index={REVIEWING_STAGGER + reviewing.length + 1}>
        <PasswordsCard />
      </FadeUp>
    </div>
  );
}
