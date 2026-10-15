import { Check } from "lucide-react";
import { motion } from "motion/react";
import { cn } from "../lib/utils";
import { nudge } from "../motion/springs";
import type { Quest } from "./QuestList";

interface QuestRowProps {
  quest: Quest;
  /** Without it the row is read-only. */
  onToggle?: ((id: string) => void) | undefined;
}

const rowClass = "flex w-full items-center gap-3 rounded-md px-3 py-2.5";

export function QuestRow({ quest, onToggle }: QuestRowProps) {
  const body = (
    <>
      <span
        aria-hidden="true"
        className={cn(
          "grid size-5 shrink-0 place-items-center rounded-full border-2 border-border",
          quest.done && "border-module bg-module text-on-accent",
        )}
      >
        {quest.done ? <Check className="size-3" strokeWidth={3} /> : null}
      </span>
      <span className={cn("flex-1 text-left", quest.done && "text-muted-foreground line-through")}>
        {quest.title}
      </span>
      <span className="font-mono text-xs text-gold">+{quest.xp} XP</span>
    </>
  );
  if (!onToggle) return <div className={rowClass}>{body}</div>;
  return (
    <motion.button
      type="button"
      role="checkbox"
      aria-checked={quest.done}
      onClick={() => {
        onToggle(quest.id);
      }}
      className={cn(rowClass, "focus-ring")}
      {...nudge}
    >
      {body}
    </motion.button>
  );
}
