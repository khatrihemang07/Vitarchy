import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import { DEFAULT_CORE } from "@/api/settings";
import { HomePage } from "./HomePage";

const now = new Date(2026, 9, 16, 18, 5);

function serve(core: object) {
  vi.stubGlobal("fetch", (request: Request) =>
    Promise.resolve(
      request.url.endsWith("/api/settings")
        ? new Response(JSON.stringify({ core: { ...DEFAULT_CORE, ...core }, modules: {} }), {
            headers: { "content-type": "application/json" },
          })
        : new Response("{}", { status: 503 }),
    ),
  );
}

function renderHome() {
  render(
    <QueryClientProvider client={new QueryClient()}>
      <HomePage moduleCount={1} now={now} />
    </QueryClientProvider>,
  );
}

afterEach(() => vi.unstubAllGlobals());

describe("HomePage header", () => {
  it("greets by display name and formats the date and time from settings", async () => {
    serve({ display_name: "Hemang", date_format: "iso", clock: "h24" });
    renderHome();
    expect(await screen.findByText("Good evening, Hemang")).toBeInTheDocument();
    expect(screen.getByText("2026-10-16 · 18:05")).toBeInTheDocument();
  });

  it("greets without a name before settings are saved", async () => {
    serve({});
    renderHome();
    expect(await screen.findByText("Good evening")).toBeInTheDocument();
  });
});
