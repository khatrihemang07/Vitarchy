import type { ComponentType } from "react";

interface FieldBase {
  /** Key inside the stored settings object. */
  readonly key: string;
  readonly label: string;
  readonly description?: string;
}

export interface SelectOption {
  readonly value: string;
  readonly label: string;
}

export interface SelectField extends FieldBase {
  readonly kind: "select";
  readonly options: readonly SelectOption[];
  readonly default: string;
}

export interface ToggleField extends FieldBase {
  readonly kind: "toggle";
  readonly default: boolean;
}

export interface TextField extends FieldBase {
  readonly kind: "text";
  readonly default: string;
  readonly maxLength?: number;
  readonly placeholder?: string;
}

export interface NumberField extends FieldBase {
  readonly kind: "number";
  readonly default: number;
  readonly min?: number;
  readonly max?: number;
}

/** One declarative setting. Add a kind here, then register its renderer. */
export type SettingField = SelectField | ToggleField | TextField | NumberField;
export type SettingKind = SettingField["kind"];
export type SettingValue = string | number | boolean;
export type SettingsValues = Record<string, SettingValue>;

export interface SettingsSection {
  /** URL segment: `/settings/{id}`. A Module's section uses the Module id. */
  readonly id: string;
  /** Where values are stored: `CORE_SCOPE`, or the owning Module's id. */
  readonly scope: string;
  readonly label: string;
  readonly icon: ComponentType<{ className?: string }>;
  readonly fields: readonly SettingField[];
}

/** What a Module declares; its section id is the Module id. */
export type ModuleSettings = Omit<SettingsSection, "id" | "scope">;

function accepts(field: SettingField, value: unknown): value is SettingValue {
  switch (field.kind) {
    case "select":
      return typeof value === "string" && field.options.some((o) => o.value === value);
    case "toggle":
      return typeof value === "boolean";
    case "text":
      return typeof value === "string";
    case "number":
      return typeof value === "number" && Number.isFinite(value);
  }
}

/** Stored values laid over the declared defaults; unknown keys and bad values are dropped. */
export function withDefaults(
  section: Pick<SettingsSection, "fields">,
  stored: Readonly<Record<string, unknown>> | undefined,
): SettingsValues {
  return Object.fromEntries(
    section.fields.map((field) => {
      const value = stored?.[field.key];
      return [field.key, accepts(field, value) ? value : field.default];
    }),
  );
}

/** The option values of a select field; empty if `key` is not a select. */
export function optionValues(section: Pick<SettingsSection, "fields">, key: string): string[] {
  const field = section.fields.find((f) => f.key === key);
  return field?.kind === "select" ? field.options.map((o) => o.value) : [];
}

type ValueOf<F extends SettingField> = F extends { kind: "select"; options: readonly (infer O)[] }
  ? O extends { value: infer V }
    ? V
    : string
  : F extends { kind: "toggle" }
    ? boolean
    : F extends { kind: "text" }
      ? string
      : number;

/**
 * Values typed from a declaration: with `as const` each key gets its own
 * type (select values become a union); a widened declaration gives the
 * general `SettingsValues`.
 */
export type ValuesOf<S extends Pick<SettingsSection, "fields">> = {
  [F in S["fields"][number] as F["key"]]: ValueOf<F>;
};
