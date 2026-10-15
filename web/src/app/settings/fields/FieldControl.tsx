import type { ComponentType } from "react";
import type { SettingField, SettingValue } from "../schema";
import { fieldRenderers } from "./fieldRenderers";

interface FieldControlProps {
  field: SettingField;
  value: SettingValue;
  onChange: (value: SettingValue) => void;
  id: string;
}

/**
 * Renders the control for `field.kind`. `withDefaults` guarantees `value`
 * matches the kind, which TypeScript cannot see across the union, so the
 * lookup is widened here, once.
 */
export function FieldControl({ field, ...rest }: FieldControlProps) {
  const Renderer = fieldRenderers[field.kind] as ComponentType<FieldControlProps>;
  return <Renderer field={field} {...rest} />;
}
