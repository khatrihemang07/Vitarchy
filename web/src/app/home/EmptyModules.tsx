import { LevelBadge, QuestList, XpBar } from "@/design";

const firstQuest = [{ id: "add-module", title: "Add your first module", xp: 50, done: false }];

/** First-run state shown while the registry is empty. */
export function EmptyModules() {
  return (
    <section
      aria-labelledby="empty-modules-title"
      className="surface-chroma animate-fade-up rounded-xl p-6"
    >
      <div className="flex items-center gap-4">
        <LevelBadge level={0} />
        <div className="min-w-0 flex-1">
          <h2 id="empty-modules-title" className="font-display text-xl font-bold">
            No modules yet
          </h2>
          <p className="text-sm text-muted-foreground">Level 0 — your first quest: add a module</p>
          <XpBar value={0} max={50} label="Level 0 progress" className="mt-2" />
        </div>
      </div>
      <QuestList quests={firstQuest} className="mt-4" />
      <p className="mt-2 text-xs text-muted-foreground">
        Modules are added in code: one folder in <code className="font-mono">src/modules</code> plus
        one registry entry. See the README.
      </p>
    </section>
  );
}
