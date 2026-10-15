import { Input } from "@/design";
import { parseNumber } from "../parseNumber";
import type { FieldProps } from "./types";
import { useDraftField } from "./useDraftField";

/** Saves on blur or Enter; text that is not a number reverts to the saved value. */
export function NumberSetting({ field, value, onChange, id }: FieldProps<"number">) {
  const inputProps = useDraftField({
    value,
    format: String,
    parse: (text) => parseNumber(text, field.min, field.max),
    onCommit: onChange,
  });
  return (
    <Input
      id={id}
      type="number"
      inputMode="decimal"
      min={field.min}
      max={field.max}
      className="md:w-32"
      {...inputProps}
    />
  );
}
