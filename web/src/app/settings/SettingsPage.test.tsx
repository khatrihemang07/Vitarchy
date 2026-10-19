import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Puzzle } from "lucide-react";
import type { ModuleManifest } from "../module-manifest";
import { createAppRouter } from "../router";
import { DEFAULT_CORE } from "@/api/settings";

const timeModule: ModuleManifest = {
  id: "time",
  label: "Time",
  accent: "#0af",
  icon: () => null,
  routes: () => [],
  navItem: { to: "/time" },
  settings: {
    label: "Time tracking",
    icon: Puzzle,
    fields: [{ kind: "toggle", key: "auto", label: "Auto-start timer", default: false }],
  },
};

type Put = { path: string; body: unknown };

/** A tiny in-memory server for /api/settings. `failPuts` makes saves answer 400. */
function serve({ failPuts = false } = {}) {
  const puts: Put[] = [];
  let state = { core: { ...DEFAULT_CORE }, modules: {} as Record<string, unknown> };
  vi.stubGlobal("fetch", async (request: Request): Promise<Response> => {
    const path = new URL(request.url).pathname;
    const reply = (status: number, body: unknown) =>
      new Response(JSON.stringify(body), {
        status,
        headers: { "content-type": "application/json" },
      });
    if (request.method === "GET") return reply(200, state);
    const body: unknown = await request.json();
    puts.push({ path, body });
    if (failPuts) return reply(400, { title: "Bad Request", status: 400, detail: "nope" });
    if (path.endsWith("/core")) state = { ...state, core: body as typeof state.core };
    else state = { ...state, modules: { ...state.modules, time: body } };
    return reply(200, body);
  });
  return puts;
}

function renderAt(path: string) {
  const queryClient = new QueryClient();
  const router = createAppRouter([timeModule], { includeDesignShowcase: false, queryClient });
  router.update({ history: createMemoryHistory({ initialEntries: [path] }), context: { queryClient } });
  render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return router;
}

afterEach(() => vi.unstubAllGlobals());

describe("Settings page", () => {
  it("redirects /settings to /settings/general", async () => {
    serve();
    const router = renderAt("/settings");
    expect(await screen.findByRole("heading", { name: "General" })).toBeInTheDocument();
    expect(router.state.location.pathname).toBe("/settings/general");
  });

  it("lists the core sections and a section declared by a Module manifest", async () => {
    serve();
    renderAt("/settings/general");
    const nav = await screen.findByRole("navigation", { name: "Settings sections" });
    expect(within(nav).getAllByRole("link").map((l) => l.textContent)).toEqual([
      "General",
      "Appearance",
      "Time tracking",
    ]);
  });

  it("renders a Module's declared fields and saves them under the Module id", async () => {
    const puts = serve();
    renderAt("/settings/time");
    await userEvent.click(await screen.findByRole("switch", { name: "Auto-start timer" }));
    expect(await screen.findByText("Saved")).toBeInTheDocument();
    expect(puts).toEqual([{ path: "/api/settings/modules/time", body: { auto: true } }]);
  });

  it("saves a core change as the full core object and shows Saved", async () => {
    const puts = serve();
    renderAt("/settings/appearance");
    await userEvent.click(await screen.findByRole("radio", { name: "Dark" }));
    expect(await screen.findByText("Saved")).toBeInTheDocument();
    expect(puts).toEqual([
      { path: "/api/settings/core", body: { ...DEFAULT_CORE, theme: "dark" } },
    ]);
  });

  it("rolls the control back and shows the error when a save is rejected", async () => {
    serve({ failPuts: true });
    renderAt("/settings/appearance");
    await userEvent.click(await screen.findByRole("radio", { name: "Dark" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Not saved. nope");
    await waitFor(() => {
      expect(
        within(screen.getByRole("radiogroup", { name: "Theme" })).getByRole("radio", { name: "System" }),
      ).toBeChecked();
    });
  });

  it("shows not-found for an unknown section", async () => {
    serve();
    renderAt("/settings/nope");
    expect(await screen.findByText(/section not found/i)).toBeInTheDocument();
  });

  it("explains when settings cannot be loaded", async () => {
    vi.stubGlobal("fetch", () => Promise.reject(new TypeError("offline")));
    renderAt("/settings/general");
    expect(await screen.findByRole("alert")).toHaveTextContent(/could not load settings/i);
  });
});
