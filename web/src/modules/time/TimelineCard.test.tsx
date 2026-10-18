import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import type { TimeStatus } from "@/api/time";

import { TimelineCard } from "./TimelineCard";
import { disconnected, jsonResponse, renderWithClient, stubApi } from "./test-utils";

afterEach(() => vi.unstubAllGlobals());

const populated: TimeStatus = {
  ...disconnected,
  timeline: {
    segment_count: 1234,
    oldest: "2026-04-01T00:00:00Z",
    newest: "2026-04-01T10:00:00Z",
    covered_secs: 36000,
    away_secs: 3600,
    unknown_secs: 18000,
  },
};

describe("TimelineCard", () => {
  it("shows nothing until the status has loaded", () => {
    stubApi({ "GET /api/time/status": () => new Promise<Response>(() => undefined) });
    const { container } = renderWithClient(<TimelineCard />);
    expect(container).toBeEmptyDOMElement();
  });

  it("invites an Import when there are no Segments", async () => {
    stubApi({ "GET /api/time/status": () => jsonResponse(200, disconnected) });
    renderWithClient(<TimelineCard />);
    expect(await screen.findByText(/No Segments yet/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Rebuild Timeline" })).toBeEnabled();
  });

  it("shows the Segment count, Away time and why Away can be unknown", async () => {
    stubApi({ "GET /api/time/status": () => jsonResponse(200, populated) });
    renderWithClient(<TimelineCard />);
    expect(await screen.findByText("1,234")).toBeInTheDocument();
    expect(screen.getByText("1h (10% of covered time)")).toBeInTheDocument();
    expect(screen.getByText("5h (50% of covered time)")).toBeInTheDocument();
    expect(screen.getByText(/no Source could tell whether you were away/)).toBeInTheDocument();
  });

  it("rebuilds, reports the result and refreshes the status", async () => {
    const fetchMock = stubApi({
      "GET /api/time/status": () => jsonResponse(200, populated),
      "POST /api/time/timeline/rebuild": () => jsonResponse(200, { activity_count: 50, segment_count: 12 }),
    });
    renderWithClient(<TimelineCard />);
    await userEvent.click(await screen.findByRole("button", { name: "Rebuild Timeline" }));

    expect(await screen.findByRole("status")).toHaveTextContent("Rebuilt: 12 Segments from 50 Activity rows.");
    await vi.waitFor(() => {
      const gets = fetchMock.mock.calls.filter(([request]) => request.method === "GET");
      expect(gets).toHaveLength(2);
    });
  });

  it("explains a failed rebuild", async () => {
    stubApi({
      "GET /api/time/status": () => jsonResponse(200, populated),
      "POST /api/time/timeline/rebuild": () => jsonResponse(500, { status: 500, detail: "x" }),
    });
    renderWithClient(<TimelineCard />);
    await userEvent.click(await screen.findByRole("button", { name: "Rebuild Timeline" }));
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });
});
