import { HOME_NAV, SETTINGS_NAV, type NavEntry } from "../nav";
import { AddModuleButton } from "./AddModuleButton";
import { Brand } from "./Brand";
import { NavItem } from "./NavItem";

interface SidebarProps {
  nav: readonly NavEntry[];
  /** Active Module id, or null when Home (or another non-Module route) is showing. */
  activeModuleId: string | null;
  homeActive: boolean;
  settingsActive: boolean;
}

/** Vertical rail on desktop; a wrapping strip along the top on narrow screens. */
export function Sidebar({ nav, activeModuleId, homeActive, settingsActive }: SidebarProps) {
  return (
    <aside className="border-b border-border/60 md:border-r md:border-b-0">
      {/* Sticky and scrollable inside the full-height aside, so the border spans the whole page. */}
      <div className="flex flex-wrap items-center gap-x-3 gap-y-2 p-3 md:sticky md:top-0 md:h-dvh md:flex-col md:flex-nowrap md:items-stretch md:gap-6 md:overflow-y-auto md:p-5">
        <Brand />
        <nav aria-label="Modules" className="flex items-center gap-1 md:flex-col md:items-stretch md:gap-1.5">
          <NavItem {...HOME_NAV} active={homeActive} />
          {nav.length > 0 ? (
            <ul className="contents">
              {nav.map(({ id, label, to, icon, accent }) => (
                <li key={id} className="contents">
                  <NavItem to={to} label={label} icon={icon} accent={accent} active={id === activeModuleId} />
                </li>
              ))}
            </ul>
          ) : null}
        </nav>
        <div className="flex items-center gap-1 max-md:ml-auto max-md:flex-row-reverse md:mt-auto md:flex-col md:items-stretch md:gap-2">
          <AddModuleButton />
          <NavItem {...SETTINGS_NAV} active={settingsActive} />
        </div>
      </div>
    </aside>
  );
}
