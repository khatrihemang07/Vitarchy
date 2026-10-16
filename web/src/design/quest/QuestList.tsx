import { cn } from "../lib/utils";
import { QuestRow } from "./QuestRow";

/** One quest: a task worth XP. */
export interface Quest {
  id: string;
  title: string;
  /** XP awarded on completion. */
  xp: number;
  done: boolean;
}

interface QuestListProps {
  quests: readonly Quest[];
  /** Makes rows toggleable; without it the list is read-only. */
  onToggle?: (id: string) => void;
  className?: string;
}

/** A checklist of quests. */
export function QuestList({ quests, onToggle, className }: QuestListProps) {
  return (
    <ul className={cn("divide-y divide-border/60", className)}>
      {quests.map((quest) => (
        <li key={quest.id}>
          <QuestRow quest={quest} onToggle={onToggle} />
        </li>
      ))}
    </ul>
  );
}
