import type { AccountSummary, SectionChoice, SectionPreview } from "@/api/finance";
import { ACCOUNT_KINDS, accountKindLabel } from "./statementCopy";

/** The fields of a new Account the person fills in. */
export interface NewAccountDraft {
  issuer: string;
  kind: string;
  name: string;
}

/** What the person is choosing for one section: an existing Account or a New Account. */
export type ChoiceDraft =
  | { mode: "existing"; accountId: number | null }
  | { mode: "new"; account: NewAccountDraft };

const isKind = (kind: string) => ACCOUNT_KINDS.some((k) => k.value === kind);

/** A name to start from: "HDFC Credit card 1234", using the last four digits only. */
export function suggestedName(section: Pick<SectionPreview, "proposed">): string {
  const { issuer, kind, mask } = section.proposed;
  const tail = mask ? mask.replace(/\D/g, "").slice(-4) : "";
  return [issuer, kind ? accountKindLabel(kind) : null, tail || null].filter(Boolean).join(" ");
}

/** Where a section's choice starts: its matched Account, or a New Account filled from what the reader found. */
export function initialDraft(section: SectionPreview): ChoiceDraft {
  if (section.matched_account) return { mode: "existing", accountId: section.matched_account.id };
  if (section.candidate_accounts.length > 0) return { mode: "existing", accountId: null };
  const { issuer, kind } = section.proposed;
  return {
    mode: "new",
    account: { issuer: issuer ?? "", kind: kind && isKind(kind) ? kind : "", name: suggestedName(section) },
  };
}

/** Whether the draft says enough to send. */
export function isDraftComplete(draft: ChoiceDraft): boolean {
  if (draft.mode === "existing") return draft.accountId !== null;
  const { issuer, kind, name } = draft.account;
  return issuer.trim() !== "" && name.trim() !== "" && isKind(kind);
}

/** The confirm request's choice for a draft. */
export function toSectionChoice(draft: ChoiceDraft): SectionChoice {
  if (draft.mode === "existing") return { account_id: draft.accountId };
  const { issuer, kind, name } = draft.account;
  return { new_account: { issuer: issuer.trim(), kind, name: name.trim() } };
}

/** The Account ids offered for a section: the ones that fit first, then every other Account. */
export function pickerAccounts(section: SectionPreview, all: readonly AccountSummary[]): { id: number; name: string }[] {
  const fitting = [...(section.matched_account ? [section.matched_account] : []), ...section.candidate_accounts];
  const fittingIds = new Set(fitting.map((a) => a.id));
  const rest = all.filter((a) => !fittingIds.has(a.id)).map((a) => ({ id: a.id, name: a.name }));
  return [...fitting.map((a) => ({ id: a.id, name: a.name })), ...rest];
}
