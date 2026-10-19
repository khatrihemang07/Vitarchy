import { useState } from "react";
import { type StatementPreview, useFinanceStatus, usePasswords, useUnconfirmedStatements } from "@/api/finance";
import { CardSkeleton, FadeUp, QueryNotices } from "@/design";
import { errorMessage } from "../failureMessage";
import { pendingTogether } from "../pendingTogether";
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

const skeleton = (
  <div className="flex flex-col gap-6">
    <CardSkeleton label="Loading the Import page" lines={2} />
    <CardSkeleton label="Loading the Import page" lines={2} />
  </div>
);

/** `/finance`: upload Statements, review and confirm them, and keep the Statement passwords. */
export function ImportPage() {
  const [reviewing, setReviewing] = useState<Reviewing[]>([]);
  const add = (items: Reviewing[]) => {
    setReviewing((current) => [...items, ...current.filter((c) => !items.some((i) => i.id === c.id))]);
  };
  // One skeleton for the whole page until its lists have answered, instead of one spinner per section.
  const loading = pendingTogether([useFinanceStatus(), useUnconfirmedStatements(), usePasswords()]);
  if (loading.isPending) return <QueryNotices query={loading} describeError={errorMessage} skeleton={skeleton} />;
  return (
    <div className="flex flex-col gap-6">
      <FadeUp index={FIRST_STAGGER}>
        <ReaderStatus />
      </FadeUp>
      <FadeUp index={FIRST_STAGGER + 1}>
        <UploadSection onUploaded={(previews) => { add(previews.map((preview) => ({ id: preview.id, preview }))); }} />
      </FadeUp>
      {reviewing.map((item) => (
        <FadeUp key={item.id} index={FIRST_STAGGER + 2}>
          {item.preview ? <StatementCard preview={item.preview} /> : <OpenedStatement id={item.id} />}
        </FadeUp>
      ))}
      <FadeUp index={FIRST_STAGGER + 2}>
        <PendingStatements excludeIds={new Set(reviewing.map((r) => r.id))} onReview={(id) => { add([{ id }]); }} />
      </FadeUp>
      <FadeUp index={FIRST_STAGGER + 3}>
        <PasswordsCard />
      </FadeUp>
    </div>
  );
}
