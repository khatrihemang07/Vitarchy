import type { AllSettings } from "@/api/settings";
import { Card } from "@/design";
import { SaveStatus } from "./SaveStatus";
import { SettingRow } from "./SettingRow";
import type { SettingsSection } from "./schema";
import { useSectionSettings } from "./useSectionSettings";

interface SettingsSectionViewProps {
  section: SettingsSection;
  settings: AllSettings;
}

/** One section: its heading, save indicator and a row per declared field. */
export function SettingsSectionView({ section, settings }: SettingsSectionViewProps) {
  const { values, save, state } = useSectionSettings(section, settings);
  return (
    <section aria-labelledby={`settings-${section.id}-title`} className="flex flex-col gap-4">
      <div className="flex min-h-8 items-center justify-between gap-4">
        <h2 id={`settings-${section.id}-title`} className="font-display text-2xl font-bold">
          {section.label}
        </h2>
        <SaveStatus state={state} />
      </div>
      <Card className="gap-0 divide-y divide-border p-6">
        {section.fields.map((field) => (
          <SettingRow
            key={field.key}
            field={field}
            idPrefix={`setting-${section.id}`}
            value={values[field.key] ?? field.default}
            onChange={(value) => {
              save(field.key, value);
            }}
          />
        ))}
      </Card>
    </section>
  );
}
