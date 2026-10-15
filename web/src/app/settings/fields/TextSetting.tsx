import { Input } from "@/design";
import { normalizeText } from "../text";
import type { FieldProps } from "./types";
import { useDraftField } from "./useDraftField";

/**
 * Saves when the user leaves the field or presses Enter, not on every keystroke.
 * Text is trimmed and limited in Unicode code points, as the server does.
 */
export function TextSetting({ field, value, onChange, id }: FieldProps<"text">) {
  const inputProps = useDraftField({
    value,
    format: (text) => text,
    parse: (text) => normalizeText(text, field.maxLength),
    sanitize: (text) => Array.from(text).slice(0, field.maxLength).join(""),
    onCommit: onChange,
  });
  return <Input id={id} placeholder={field.placeholder} className="md:w-72" {...inputProps} />;
}
