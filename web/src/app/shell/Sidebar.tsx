import { HOME_NAV, type NavEntry } from "../nav";
import { AddModuleButton } from "./AddModuleButton";
import { Brand } from "./Brand";
import { NavItem } from "./NavItem";

interface SidebarProps {
  nav: readonly NavEntry[];
  /** Active Module id, or null when Home (or another non-Module route) is showing. */
  activeModuleId: string | null;
  homeActive: boolean;
}

/** Vertical rail on desktop; a horizontally scrolling strip on narrow screens. */
export function Sidebar({ nav, activeModuleId, homeActive }: SidebarProps) {
  return (
    <aside className="flex items-center gap-3 overflow-x-auto border-b border-border/60 p-3 md:sticky md:top-0 md:h-dvh md:flex-col md:items-stretch md:gap-6 md:overflow-visible md:border-r md:border-b-0 md:p-5">
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
      <div className="md:mt-auto">
        <AddModuleButton />
      </div>
    </aside>
  );
}
