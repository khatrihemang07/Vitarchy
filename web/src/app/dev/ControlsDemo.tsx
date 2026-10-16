import { Plus } from "lucide-react";
import { Badge, Button, MotionButton, Separator } from "@/design";

/** Buttons and badges in each variant. */
export function ControlsDemo() {
  return (
    <div className="flex flex-wrap items-center gap-3">
      <MotionButton>
        <Plus /> Primary
      </MotionButton>
      <MotionButton variant="secondary">Secondary</MotionButton>
      <Button variant="outline">Outline</Button>
      <Button variant="ghost">Ghost</Button>
      <Button variant="destructive">Destructive</Button>
      <Button disabled>Disabled</Button>
      <Separator orientation="vertical" className="h-6" />
      <Badge>Badge</Badge>
      <Badge variant="outline">Outline</Badge>
    </div>
  );
}
