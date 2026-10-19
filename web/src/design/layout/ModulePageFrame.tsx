import type { ReactNode } from "react";
import { FadeUp } from "../motion/FadeUp";
import { ModulePageHeader } from "./ModulePageHeader";
import { type ModuleTab, ModuleTabs } from "./ModuleTabs";

interface ModulePageFrameProps {
  title: string;
  subtitle: string;
  tabs: readonly ModuleTab[];
  /** Accessible name of the tab navigation. */
  tabsLabel: string;
  /** Layout id the active tab pill glides with; unique per Module. */
  indicatorId: string;
  /** The page itself, usually the router's `<Outlet />`. */
  children: ReactNode;
}

/**
 * The frame every page of a Module shares: header, tabs, then the page. It fills the width the
 * shell gives it, so no page sets a width of its own.
 */
export function ModulePageFrame({ title, subtitle, tabs, tabsLabel, indicatorId, children }: ModulePageFrameProps) {
  return (
    <div className="flex flex-col gap-6">
      <ModulePageHeader title={title} subtitle={subtitle} />
      <FadeUp index={1}>
        <ModuleTabs tabs={tabs} label={tabsLabel} indicatorId={indicatorId} />
      </FadeUp>
      {children}
    </div>
  );
}
