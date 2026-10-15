import { Link } from "@tanstack/react-router";
import { useSettings } from "@/api/settings";
import { Button } from "@/design";
import { SettingsNotice } from "./SettingsNotice";
import { SettingsSectionView } from "./SettingsSectionView";
import type { SettingsSection } from "./schema";

/** The active section, or the notice that explains why there is none. */
export function SettingsContent({ section }: { section: SettingsSection | undefined }) {
  const settings = useSettings();

  if (!section) {
    return (
      <SettingsNotice>
        <p className="font-medium text-foreground">Settings section not found.</p>
        <Button asChild variant="link" className="px-0">
          <Link to="/settings/$section" params={{ section: "general" }}>
            Back to General
          </Link>
        </Button>
      </SettingsNotice>
    );
  }
  if (settings.data) {
    return <SettingsSectionView key={section.id} section={section} settings={settings.data} />;
  }
  if (settings.isError) {
    return (
      <SettingsNotice>
        <p role="alert" className="font-medium text-foreground">
          Could not load settings. Is the server running?
        </p>
        <Button
          variant="outline"
          onClick={() => {
            void settings.refetch();
          }}
        >
          Try again
        </Button>
      </SettingsNotice>
    );
  }
  return <SettingsNotice>Loading settings…</SettingsNotice>;
}
