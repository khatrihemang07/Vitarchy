import type { ReactNode } from "react";
import { stagger } from "./stagger";

/** Fades its children up as one step of a page's entrance sequence; `index` is its place in the order. */
export function FadeUp({ index, children }: { index: number; children: ReactNode }) {
  return (
    <div className="animate-fade-up" style={stagger(index)}>
      {children}
    </div>
  );
}
