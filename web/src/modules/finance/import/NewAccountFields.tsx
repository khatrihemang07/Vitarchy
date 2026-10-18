import { useId } from "react";
import { Input, Label, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/design";
import type { NewAccountDraft } from "../sectionChoice";
import { ACCOUNT_KINDS } from "../statementCopy";

interface NewAccountFieldsProps {
  value: NewAccountDraft;
  onChange: (next: NewAccountDraft) => void;
}

/** The name, issuer and kind of a New Account. */
export function NewAccountFields({ value, onChange }: NewAccountFieldsProps) {
  const id = useId();
  return (
    <div className="grid gap-3 sm:grid-cols-3">
      <div className="flex flex-col gap-1.5">
        <Label htmlFor={`${id}-name`}>Name</Label>
        <Input id={`${id}-name`} value={value.name} onChange={(e) => { onChange({ ...value, name: e.target.value }); }} />
      </div>
      <div className="flex flex-col gap-1.5">
        <Label htmlFor={`${id}-issuer`}>Issuer</Label>
        <Input id={`${id}-issuer`} value={value.issuer} onChange={(e) => { onChange({ ...value, issuer: e.target.value }); }} />
      </div>
      <div className="flex flex-col gap-1.5">
        <Label id={`${id}-kind`}>Kind</Label>
        <Select value={value.kind} onValueChange={(kind) => { onChange({ ...value, kind }); }}>
          <SelectTrigger className="w-full" aria-labelledby={`${id}-kind`}>
            <SelectValue placeholder="Choose a kind" />
          </SelectTrigger>
          <SelectContent>
            {ACCOUNT_KINDS.map((kind) => (
              <SelectItem key={kind.value} value={kind.value}>{kind.label}</SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
    </div>
  );
}
