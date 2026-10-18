/** Public API of the design system. Import from "@/design", never deep paths. */
import "./theme/fonts";
import "./theme/styles.css";

export { accentScope } from "./lib/accent-scope";
export { cn } from "./lib/utils";

export { LogoMark } from "./brand/LogoMark";

export * from "./ui/badge";
export * from "./ui/button";
export * from "./ui/card";
export * from "./ui/command";
export * from "./ui/dialog";
export * from "./ui/input";
export * from "./ui/label";
export * from "./ui/popover";
export * from "./ui/radio-group";
export * from "./ui/select";
export * from "./ui/separator";
export * from "./ui/switch";
export * from "./ui/tooltip";

export { DailyBars, type DailyBarPoint } from "./charts/DailyBars";
export { DailyStackedBars, type StackedDayPoint } from "./charts/DailyStackedBars";
export { HourHeat } from "./charts/HourHeat";
export { SegmentedControl, type SegmentedOption } from "./controls/SegmentedControl";

export * from "./effects/border-beam";
export * from "./effects/magic-card";
export * from "./effects/number-ticker";

export { MotionButton } from "./motion/MotionButton";
export { Expand } from "./motion/Expand";
export { FadeUp } from "./motion/FadeUp";
export { FadeValue } from "./motion/FadeValue";
export { MotionProvider } from "./motion/MotionProvider";
export { Pressable } from "./motion/Pressable";
export { nudge, pressable, springs } from "./motion/springs";
export { stagger } from "./motion/stagger";
export { useMotionReduced } from "./motion/useMotionReduced";

export { AchievementBadge } from "./quest/AchievementBadge";
export { LevelBadge } from "./quest/LevelBadge";
export { QuestList, type Quest } from "./quest/QuestList";
export { StatCard } from "./quest/StatCard";
export { StreakFlame } from "./quest/StreakFlame";
export { XpBar } from "./quest/XpBar";
