import type { ComponentType } from "react";
import type { SettingField, SettingKind } from "../schema";

type FieldOf<K extends SettingKind> = Extract<SettingField, { kind: K }>;

/** Props every field renderer receives; `V` is the kind's value type. */
export interface FieldProps<K extends SettingKind> {
  field: FieldOf<K>;
  value: FieldOf<K>["default"];
  /** Called with the new value when the user commits a change. */
  onChange: (value: FieldOf<K>["default"]) => void;
  /** Id the row's <label> points at. */
  id: string;
}

/** Maps each field kind to the component that renders it. */
export type FieldRenderers = { [K in SettingKind]: ComponentType<FieldProps<K>> };
