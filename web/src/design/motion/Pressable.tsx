import { motion, type HTMLMotionProps } from "motion/react";
import { pressable } from "./springs";

/** Any block that should spring on hover and press (cards, tiles, nav items). */
export function Pressable(props: HTMLMotionProps<"div">) {
  return <motion.div {...pressable} {...props} />;
}
