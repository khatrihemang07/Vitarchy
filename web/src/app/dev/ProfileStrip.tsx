import { AchievementBadge, LevelBadge, StreakFlame, XpBar } from "@/design";
import { achievements, profile } from "./sample-data";

/** XP bar, level, streak and achievements together. */
export function ProfileStrip() {
  return (
    <div className="flex flex-wrap items-center gap-x-6 gap-y-3 rounded-lg border-2 border-border bg-card p-4">
      <LevelBadge level={profile.level} />
      <div className="min-w-44 flex-1">
        <div className="flex justify-between gap-2 text-xs text-muted-foreground">
          <b className="text-sm text-foreground">
            {profile.name}, level {profile.level}
          </b>
          <span>
            {profile.xp.toLocaleString()} / {profile.nextLevelXp.toLocaleString()} XP
          </span>
        </div>
        <XpBar
          value={profile.xp}
          max={profile.nextLevelXp}
          label={`Level ${String(profile.level)} progress`}
          className="mt-1.5"
        />
      </div>
      <StreakFlame days={profile.streakDays} />
      <div className="flex gap-1.5" role="group" aria-label="Achievements">
        {achievements.map((a) => (
          <AchievementBadge key={a.label} {...a} />
        ))}
      </div>
    </div>
  );
}
