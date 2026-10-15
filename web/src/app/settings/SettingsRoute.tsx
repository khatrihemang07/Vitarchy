import { useParams } from "@tanstack/react-router";
import { SettingsPage } from "./SettingsPage";
import type { SettingsSection } from "./schema";

/** Binds the `$section` URL param to the page. */
export function SettingsRoute({ sections }: { sections: readonly SettingsSection[] }) {
  const section = useParams({
    strict: false,
    select: (params: { section?: string }) => params.section,
  });
  return <SettingsPage sections={sections} sectionId={section ?? ""} />;
}
