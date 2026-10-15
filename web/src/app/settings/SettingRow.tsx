import { Label } from "@/design";
import { FieldControl } from "./fields/FieldControl";
import type { SettingField, SettingValue } from "./schema";

interface SettingRowProps {
  field: SettingField;
  value: SettingValue;
  onChange: (value: SettingValue) => void;
  /** Unique prefix so ids stay distinct across sections. */
  idPrefix: string;
}

/** Label and description on one side, the field's control on the other. */
export function SettingRow({ field, value, onChange, idPrefix }: SettingRowProps) {
  const id = `${idPrefix}-${field.key}`;
  return (
    <div className="flex flex-col gap-3 py-4 first:pt-0 last:pb-0 md:flex-row md:items-center md:justify-between md:gap-8">
      <div className="min-w-0">
        <Label htmlFor={id} className="text-base font-medium">
          {field.label}
        </Label>
        {field.description ? (
          <p className="text-sm text-muted-foreground">{field.description}</p>
        ) : null}
      </div>
      <FieldControl field={field} value={value} onChange={onChange} id={id} />
    </div>
  );
}
