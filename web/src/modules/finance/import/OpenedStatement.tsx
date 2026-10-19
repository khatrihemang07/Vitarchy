import { useStatementPreview } from "@/api/finance";
import { CardSkeleton, QueryNotices } from "@/design";
import { errorMessage } from "../failureMessage";
import { StatementCard } from "./StatementCard";

/** Loads one stored Statement's preview and shows it as a card. */
export function OpenedStatement({ id }: { id: number }) {
  const preview = useStatementPreview(id, true);
  if (!preview.data) {
    return <QueryNotices query={preview} describeError={errorMessage} skeleton={<CardSkeleton label="Opening the Statement…" lines={4} />} />;
  }
  return <StatementCard preview={preview.data} />;
}
