import { ControlsDemo } from "./ControlsDemo";
import { FeedbackDemo } from "./FeedbackDemo";
import { ModuleCards } from "./ModuleCards";
import { ProfileStrip } from "./ProfileStrip";
import { QuestDemo } from "./QuestDemo";
import { Section } from "./Section";

/** Dev-only gallery of every design primitive, with sample data. Route: /design. */
export function DesignShowcase() {
  return (
    <div className="flex flex-col gap-8">
      <header>
        <h1 className="font-display text-4xl font-bold tracking-tight">Design system</h1>
        <p className="text-muted-foreground">Chroma + Quest. Sample data, dev builds only.</p>
      </header>
      <Section title="Profile (XpBar, LevelBadge, StreakFlame, AchievementBadge)">
        <ProfileStrip />
      </Section>
      <Section title="Module accents (StatCard, NumberTicker, MagicCard, BorderBeam)">
        <ModuleCards />
      </Section>
      <Section title="Quest log (QuestList)">
        <QuestDemo />
      </Section>
      <Section title="Controls (shadcn, Motion springs)">
        <ControlsDemo />
      </Section>
      <Section title="Feedback (Notice, Progress)">
        <FeedbackDemo />
      </Section>
    </div>
  );
}
