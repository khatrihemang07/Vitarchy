import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRoute,
  RouterProvider,
} from "@tanstack/react-router";
import { render, screen, within } from "@testing-library/react";
import type { ModuleManifest } from "./module-manifest";
import { createAppRouter } from "./router";

const timeModule: ModuleManifest = {
  id: "time",
  label: "Time",
  accent: "#0af",
  icon: () => null,
  routes: (root) => [
    createRoute({
      getParentRoute: () => root,
      path: "/time",
      component: () => <p>time page</p>,
    }),
  ],
  navItem: { to: "/time" },
};

function renderAt(path: string, modules: ModuleManifest[], includeDesignShowcase = false) {
  const router = createAppRouter(modules, { includeDesignShowcase });
  router.update({ history: createMemoryHistory({ initialEntries: [path] }) });
  return render(
    <QueryClientProvider client={new QueryClient()}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
}

describe("app router", () => {
  beforeEach(() => {
    vi.stubGlobal("fetch", () => Promise.reject(new Error("offline")));
  });
  afterEach(() => vi.unstubAllGlobals());

  it("renders no module links for an empty registry", async () => {
    renderAt("/", []);
    await screen.findByRole("heading", { name: "Home" });
    const nav = screen.getByRole("navigation", { name: "Modules" });
    expect(within(nav).queryAllByRole("listitem")).toHaveLength(0);
    expect(within(nav).getByRole("link", { name: "Home" })).toHaveAttribute("href", "/");
  });

  it("shows a disabled Add module affordance", async () => {
    renderAt("/", []);
    expect(await screen.findByRole("button", { name: /add module/i })).toBeDisabled();
  });

  it("exposes the active Module accent on the shell", async () => {
    const { container } = renderAt("/time", [timeModule]);
    await screen.findByText("time page");
    const shell = container.querySelector("[data-module]");
    expect(shell).toHaveAttribute("data-module", "time");
    expect(shell).toHaveStyle({ "--module-accent": "#0af" });
  });

  it("uses the brand accent on Home", async () => {
    const { container } = renderAt("/", [timeModule]);
    await screen.findByRole("heading", { name: "Home" });
    const shell = container.querySelector("[data-module]");
    expect(shell).toHaveAttribute("data-module", "home");
    expect(shell).toHaveStyle({ "--module-accent": "var(--brand)" });
  });

  it("registers /design only when the showcase is enabled", async () => {
    renderAt("/design", [], true);
    expect(await screen.findByRole("heading", { name: "Design system" })).toBeInTheDocument();
  });

  it("has no /design route when the showcase is disabled", async () => {
    renderAt("/design", [], false);
    expect(await screen.findByText(/not found/i)).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "Design system" })).not.toBeInTheDocument();
  });

  it("builds nav links and routes from the registry", async () => {
    renderAt("/time", [timeModule]);
    expect(await screen.findByText("time page")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /Time/ })).toHaveAttribute("href", "/time");
  });
});
