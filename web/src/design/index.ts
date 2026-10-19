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
export { ModulePageFrame } from "./layout/ModulePageFrame";
export { ModulePageHeader } from "./layout/ModulePageHeader";
export { ModuleTabs, type ModuleTab } from "./layout/ModuleTabs";
export { ListRow } from "./layout/ListRow";
export { SectionCard } from "./layout/SectionCard";
export { IconChip } from "./notice/IconChip";
export { Notice } from "./notice/Notice";
export { NOTICE_VIEWS, WARNING_STYLE, type NoticeKind, type NoticeView } from "./notice/noticeViews";
export { QueryNotices } from "./notice/QueryNotices";
export { Progress } from "./progress/Progress";
export { CardSkeleton } from "./skeleton/CardSkeleton";
export { Skeleton } from "./skeleton/Skeleton";
export { TableSkeleton } from "./skeleton/TableSkeleton";
export { ColumnHeaderBar, type ColumnHeading } from "./table/ColumnHeaderBar";
export { ClearFiltersButton } from "./controls/ClearFiltersButton";
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
