import { m } from "motion/react";
import { useState } from "react";
import { cn } from "../lib/utils";
import { valueFade } from "./springs";
import { useMotionReduced } from "./useMotionReduced";

interface FadeValueProps {
  /** The formatted value; a change of text is what triggers the fade. */
  text: string;
  className?: string;
}

/**
 * A figure that fades in softly when its text changes, instead of counting up. It stays
 * still on first mount (the section's own entrance covers that) and is a plain span when
 * motion is reduced.
 */
export function FadeValue({ text, className }: FadeValueProps) {
  const reduced = useMotionReduced();
  // Adjusting state while rendering (React's documented pattern): the first change of text
  // switches fading on, so the value that mounted with the page never fades.
  const [shown, setShown] = useState(text);
  const [changed, setChanged] = useState(false);
  if (shown !== text) {
    setShown(text);
    setChanged(true);
  }
  const classes = cn("tabular-nums", className);
  if (reduced) return <span className={classes}>{text}</span>;
  return (
    <m.span
      key={text}
      className={classes}
      initial={changed ? { opacity: valueFade.from } : false}
      animate={{ opacity: 1 }}
      transition={valueFade.transition}
    >
      {text}
    </m.span>
  );
}
