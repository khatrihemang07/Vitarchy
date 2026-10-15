// Sample data for the /design showcase only. Never used by real screens.
import type { Quest } from "@/design";

export const profile = { name: "Hemang", level: 12, xp: 2340, nextLevelXp: 3000, streakDays: 9 };

export const quests: Quest[] = [
  { id: "1", title: "Log 2 hours of deep work", xp: 40, done: true },
  { id: "2", title: "Review this week's budget", xp: 30, done: false },
  { id: "3", title: "Plan tomorrow", xp: 20, done: false },
];

export const achievements = [
  { glyph: "E", label: "Early bird", tone: "gold" },
  { glyph: "S", label: "Saver", tone: "mint" },
  { glyph: "F", label: "Focused", tone: "sky" },
  { glyph: "?", label: "Locked achievement", tone: "locked" },
] as const;
