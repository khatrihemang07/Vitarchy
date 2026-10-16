import {
  createRootRoute,
  createRoute,
  createRouter,
  lazyRouteComponent,
  redirect,
} from "@tanstack/react-router";
import { HomePage } from "./home/HomePage";
import { NotFound } from "./NotFound";
import type { ModuleManifest } from "./module-manifest";
import { buildNav } from "./nav";
import { generalSection } from "./settings/core-sections";
import { SettingsRoute } from "./settings/SettingsRoute";
import { buildSections } from "./settings/sections";
import { Shell } from "./shell/Shell";

/** Folded to `undefined` in production builds, so the showcase is not even bundled. */
const loadShowcase = import.meta.env.DEV ? () => import("./dev/DesignShowcase") : undefined;

interface RouterOptions {
  /** Registers the /design showcase. Defaults to dev builds only. */
  includeDesignShowcase?: boolean;
}

/** Builds the router: Home, Settings, the dev showcase, then each Module's routes. */
export function createAppRouter(
  modules: readonly ModuleManifest[],
  { includeDesignShowcase = import.meta.env.DEV }: RouterOptions = {},
) {
  const nav = buildNav(modules);
  const sections = buildSections(modules);
  const root = createRootRoute({ component: () => <Shell nav={nav} sections={sections} /> });
  const home = createRoute({
    getParentRoute: () => root,
    path: "/",
    component: () => <HomePage moduleCount={modules.length} />,
  });
  const settingsIndex = createRoute({
    getParentRoute: () => root,
    path: "/settings",
    beforeLoad: () => {
      // eslint-disable-next-line @typescript-eslint/only-throw-error -- TanStack Router's redirect() is thrown by design.
      throw redirect({ to: "/settings/$section", params: { section: sections[0]?.id ?? generalSection.id } });
    },
  });
  const settings = createRoute({
    getParentRoute: () => root,
    path: "/settings/$section",
    component: () => <SettingsRoute sections={sections} />,
  });
  const design =
    includeDesignShowcase && loadShowcase
      ? [
          createRoute({
            getParentRoute: () => root,
            path: "/design",
            component: lazyRouteComponent(loadShowcase, "DesignShowcase"),
          }),
        ]
      : [];
  return createRouter({
    defaultNotFoundComponent: NotFound,
    routeTree: root.addChildren([
      home,
      settingsIndex,
      settings,
      ...design,
      ...modules.flatMap((m) => m.routes(root)),
    ]),
  });
}
