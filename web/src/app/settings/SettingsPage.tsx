import { SectionNav } from "./SectionNav";
import { SettingsContent } from "./SettingsContent";
import type { SettingsSection } from "./schema";

interface SettingsPageProps {
  sections: readonly SettingsSection[];
  /** The `$section` URL segment. */
  sectionId: string;
}

/** `/settings/$section`: the section list beside (or above) the active section. */
export function SettingsPage({ sections, sectionId }: SettingsPageProps) {
  return (
    <div className="flex flex-col gap-6">
      <header className="animate-fade-up">
        <h1 className="font-display text-4xl font-bold tracking-tight">Settings</h1>
      </header>
      <div className="flex flex-col gap-6 md:flex-row md:gap-10">
        <SectionNav sections={sections} activeId={sectionId} />
        <div className="min-w-0 flex-1">
          <SettingsContent section={sections.find((s) => s.id === sectionId)} />
        </div>
      </div>
    </div>
  );
}
