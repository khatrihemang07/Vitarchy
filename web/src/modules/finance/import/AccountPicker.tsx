import { useId } from "react";
import type { AccountSummary, SectionPreview } from "@/api/finance";
import { Label, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/design";
import { type ChoiceDraft, initialDraft, pickerAccounts } from "../sectionChoice";
import { NewAccountFields } from "./NewAccountFields";

const NEW = "new";
const accountValue = (id: number) => `account:${String(id)}`;

interface AccountPickerProps {
  section: SectionPreview;
  /** Every Account, for choosing one the reader did not match. */
  accounts: readonly AccountSummary[];
  draft: ChoiceDraft;
  onChange: (next: ChoiceDraft) => void;
}

/** Chooses the Account a section belongs to: the matched one, any existing one, or a New Account. */
export function AccountPicker({ section, accounts, draft, onChange }: AccountPickerProps) {
  const id = useId();
  const options = pickerAccounts(section, accounts);
  const value = draft.mode === "new" ? NEW : draft.accountId === null ? "" : accountValue(draft.accountId);

  const choose = (next: string) => {
    if (next === NEW) {
      const fresh = initialDraft({ ...section, matched_account: null, candidate_accounts: [] });
      onChange(draft.mode === "new" ? draft : fresh);
    } else {
      onChange({ mode: "existing", accountId: Number(next.replace("account:", "")) });
    }
  };

  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-col gap-1.5">
        <Label id={`${id}-account`}>Account</Label>
        <Select value={value} onValueChange={choose}>
          <SelectTrigger className="w-full sm:w-80" aria-labelledby={`${id}-account`}>
            <SelectValue placeholder="Choose an Account" />
          </SelectTrigger>
          <SelectContent>
            {options.map((option) => (
              <SelectItem key={option.id} value={accountValue(option.id)}>{option.name}</SelectItem>
            ))}
            <SelectItem value={NEW}>New Account</SelectItem>
          </SelectContent>
        </Select>
        {section.matched_without_number && draft.mode === "existing" && draft.accountId === section.matched_account?.id ? (
          <p className="text-sm text-muted-foreground">No account number printed — matched by bank and kind</p>
        ) : null}
      </div>
      {draft.mode === "new" ? (
        <NewAccountFields value={draft.account} onChange={(account) => { onChange({ mode: "new", account }); }} />
      ) : null}
    </div>
  );
}
