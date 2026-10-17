import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createMemoryHistory,
  createRoute,
  RouterProvider,
} from "@tanstack/react-router";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Wallet } from "lucide-react";
import type { ModuleManifest } from "./module-manifest";
import { createAppRouter } from "./router";
import { DEFAULT_CORE } from "@/api/settings";

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

const financeModule: ModuleManifest = {
  id: "finance",
  label: "Finance",
  accent: "#0a8",
  icon: () => null,
  routes: () => [],
  navItem: { to: "/finance" },
  settings: {
    label: "Finance",
    icon: Wallet,
    fields: [{ kind: "toggle", key: "roundUp", label: "Round up savings", default: false }],
  },
};

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
    expect(await screen.findByRole("heading", { name: "Page not found" })).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "Design system" })).not.toBeInTheDocument();
  });

  it("links Settings from the sidebar", async () => {
    renderAt("/", []);
    expect(await screen.findByRole("link", { name: "Settings" })).toHaveAttribute(
      "href",
      "/settings/general",
    );
  });

  it("lists Settings and each section in the command bar", async () => {
    renderAt("/", []);
    await userEvent.click(await screen.findByRole("button", { name: /quick add/i }));
    expect(await screen.findByRole("option", { name: "Settings" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "Settings: Appearance" })).toBeInTheDocument();
  });

  it("opens a settings section from the command bar", async () => {
    renderAt("/", []);
    await userEvent.click(await screen.findByRole("button", { name: /quick add/i }));
    await userEvent.click(await screen.findByRole("option", { name: "Settings: Appearance" }));
    await waitFor(() => {
      expect(screen.getByRole("navigation", { name: "Settings sections" })).toBeInTheDocument();
    });
  });

  it("builds nav links and routes from the registry", async () => {
    renderAt("/time", [timeModule]);
    expect(await screen.findByText("time page")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /Time/ })).toHaveAttribute("href", "/time");
  });

  it("shows a Module's declared settings in the Settings nav and at /settings/<id>", async () => {
    const reply = (body: unknown) =>
      new Response(JSON.stringify(body), { headers: { "content-type": "application/json" } });
    vi.stubGlobal("fetch", () =>
      Promise.resolve(reply({ core: DEFAULT_CORE, modules: { finance: { roundUp: true } } })),
    );
    renderAt("/settings/finance", [timeModule, financeModule]);

    const nav = await screen.findByRole("navigation", { name: "Settings sections" });
    // Only Modules that declare `settings` get a section.
    expect(within(nav).getAllByRole("link").map((l) => l.textContent)).toEqual([
      "General",
      "Appearance",
      "Finance",
    ]);
    expect(await screen.findByRole("heading", { name: "Finance" })).toBeInTheDocument();
    expect(await screen.findByRole("switch", { name: "Round up savings" })).toBeChecked();
  });
});

describe("unknown URL", () => {
  it("shows a Page not found state with a way back Home", async () => {
    renderAt("/nope/never", []);
    expect(await screen.findByRole("heading", { name: "Page not found" })).toBeInTheDocument();
    expect(screen.getByText(/nothing lives at this address/i)).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Back to Home" })).toHaveAttribute("href", "/");
  });

  it("logs a scoped warning with the path", async () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => undefined);
    renderAt("/nope/never", []);
    await waitFor(() => {
      expect(warn).toHaveBeenCalledWith("[router] route not found", { path: "/nope/never" });
    });
    warn.mockRestore();
  });
});
