import type { ReactNode } from "react";
import { Card } from "@/design";

/** A card for states with no settings to show: loading, failure, unknown section. */
export function SettingsNotice({ children }: { children: ReactNode }) {
  return <Card className="items-start p-6 text-muted-foreground">{children}</Card>;
}
