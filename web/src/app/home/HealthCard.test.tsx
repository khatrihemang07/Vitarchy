import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import { HealthCard } from "./HealthCard";

function stubFetch(impl: () => Promise<Response>) {
  vi.stubGlobal("fetch", vi.fn(impl));
}

function json(status: number, body: unknown) {
  return Promise.resolve(
    new Response(JSON.stringify(body), {
      status,
      headers: { "content-type": "application/json" },
    }),
  );
}

function renderCard() {
  render(
    <QueryClientProvider client={new QueryClient()}>
      <HealthCard />
    </QueryClientProvider>,
  );
}

describe("HealthCard", () => {
  afterEach(() => vi.unstubAllGlobals());

  it("shows ok when the server and database are healthy", async () => {
    stubFetch(() => json(200, { status: "ok", db: "ok" }));
    renderCard();
    expect(await screen.findByText(/database ok/i)).toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveAttribute("data-state", "ok");
  });

  it("shows database down on a 503", async () => {
    stubFetch(() => json(503, { status: "degraded", db: "down" }));
    renderCard();
    expect(await screen.findByText(/database down/i)).toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveAttribute("data-state", "degraded");
  });

  it("shows unreachable when the request fails", async () => {
    stubFetch(() => Promise.reject(new TypeError("Failed to fetch")));
    renderCard();
    expect(await screen.findByText(/unreachable/i)).toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveAttribute("data-state", "offline");
  });
});
