import { AnimatePresence, m } from "motion/react";
import type { ReactNode } from "react";
import { springs } from "./springs";
import { useMotionReduced } from "./useMotionReduced";

interface ExpandProps {
  open: boolean;
  id?: string;
  children: ReactNode;
}

/** The panel takes its parent's corner radius, so a rounded parent's last row keeps its corners without clipping. */
const INHERIT_CORNERS = { borderRadius: "inherit" } as const;

/**
 * Reveals its children by animating height. The `id` element stays mounted, so a trigger's
 * `aria-controls` always names a real element. Overflow is clipped only while the height
 * moves, so tooltips and focus rings inside are never cut off at rest. When motion is
 * reduced it skips the animation: MotionConfig does not cover height or opacity.
 */
export function Expand({ open, id, children }: ExpandProps) {
  const reduced = useMotionReduced();
  if (reduced) return <div id={id} style={INHERIT_CORNERS}>{open ? children : null}</div>;
  return (
    <div id={id} style={INHERIT_CORNERS}>
      <AnimatePresence initial={false}>
        {open ? (
          <m.div
            style={INHERIT_CORNERS}
            initial={{ height: 0, opacity: 0, overflow: "hidden" }}
            animate={{ height: "auto", opacity: 1, transitionEnd: { overflow: "visible" } }}
            exit={{ height: 0, opacity: 0, overflow: "hidden" }}
            transition={springs.smooth}
          >
            {children}
          </m.div>
        ) : null}
      </AnimatePresence>
    </div>
  );
}
