import {
  createRootRoute,
  createRoute,
  createRouter,
  lazyRouteComponent,
} from "@tanstack/react-router";
import { HomePage } from "./home/HomePage";
import type { ModuleManifest } from "./module-manifest";
import { buildNav } from "./nav";
import { Shell } from "./shell/Shell";

/** Folded to `undefined` in production builds, so the showcase is not even bundled. */
const loadShowcase = import.meta.env.DEV ? () => import("./dev/DesignShowcase") : undefined;

interface RouterOptions {
  /** Registers the /design showcase. Defaults to dev builds only. */
  includeDesignShowcase?: boolean;
}

export function createAppRouter(
  modules: readonly ModuleManifest[],
  { includeDesignShowcase = import.meta.env.DEV }: RouterOptions = {},
) {
  const nav = buildNav(modules);
  const root = createRootRoute({ component: () => <Shell nav={nav} /> });
  const home = createRoute({
    getParentRoute: () => root,
    path: "/",
    component: () => <HomePage moduleCount={modules.length} />,
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
    routeTree: root.addChildren([
      home,
      ...design,
      ...modules.flatMap((m) => m.routes(root)),
    ]),
  });
}
