import { screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { jsonResponse, renderWithClient, stubApi } from "../test-utils";
import { LIFELOG_OVERVIEW, LOGGED, lifeLogBody } from "./overviewFixture";
import { TotalsTable } from "./TotalsTable";

const period = { from: "2026-05-04", to: "2026-05-05", tz: "UTC" };

afterEach(() => vi.unstubAllGlobals());

describe("TotalsTable", () => {
  it("lists projects with time, entries, days and share, and opens one to show its descriptions", async () => {
    stubApi({ [LIFELOG_OVERVIEW]: () => jsonResponse(200, LOGGED) });
    renderWithClient(<TotalsTable period={period} />);
    const office = (await screen.findByRole("button", { name: /office/ })).closest("tr");
    expect(office).not.toBeNull();
    expect(within(office as HTMLElement).getByText("8 h")).toBeInTheDocument();
    expect(within(office as HTMLElement).getByText("47%")).toBeInTheDocument();
    expect(screen.queryByText("inside_office")).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: /office/ }));
    expect(screen.getByRole("button", { name: /office/ })).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByText("inside_office")).toBeInTheDocument();
  });

  it("switches to descriptions across projects", async () => {
    stubApi({ [LIFELOG_OVERVIEW]: () => jsonResponse(200, LOGGED) });
    renderWithClient(<TotalsTable period={period} />);
    await userEvent.click(await screen.findByRole("radio", { name: "By description" }));
    expect(screen.getByRole("columnheader", { name: "Description" })).toBeInTheDocument();
    expect(screen.getByText("inside_office")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /office/ })).not.toBeInTheDocument();
  });

  it("says so when nothing is logged, and explains a failure", async () => {
    stubApi({ [LIFELOG_OVERVIEW]: () => jsonResponse(200, lifeLogBody()) });
    const { unmount } = renderWithClient(<TotalsTable period={period} />);
    expect(await screen.findByText("No Time Entries in this Period.")).toBeInTheDocument();
    unmount();
    stubApi({ [LIFELOG_OVERVIEW]: () => jsonResponse(500, { detail: "boom" }) });
    renderWithClient(<TotalsTable period={period} />);
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });
});
