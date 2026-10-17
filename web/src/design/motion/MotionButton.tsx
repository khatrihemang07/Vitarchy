import { m } from "motion/react";
import type { ComponentProps } from "react";
import { Button } from "../ui/button";
import { pressable } from "./springs";

type MotionButtonProps = Omit<ComponentProps<typeof Button>, "asChild">;

/** The shadcn Button with the shared hover/press spring (disabled buttons ignore the pointer). */
export function MotionButton({ children, ...props }: MotionButtonProps) {
  return (
    <Button asChild {...props}>
      <m.button {...pressable}>{children}</m.button>
    </Button>
  );
}
