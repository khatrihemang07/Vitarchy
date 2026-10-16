import { Switch } from "@/design";
import type { FieldProps } from "./types";

/** An on/off switch. */
export function ToggleSetting({ value, onChange, id }: FieldProps<"toggle">) {
  return <Switch id={id} checked={value} onCheckedChange={onChange} />;
}
