import { Plus } from "lucide-react";
import { Button } from "@/design";

/** Placeholder until Modules can be installed from the UI. */
export function AddModuleButton() {
  return (
    <Button
      variant="outline"
      disabled
      className="shrink-0 rounded-full border-dashed text-muted-foreground"
    >
      <Plus />
      Add module
    </Button>
  );
}
