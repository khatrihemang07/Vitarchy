import { screen } from "@testing-library/react";
import { jsonResponse, renderWithClient, stubApi } from "../../test-utils";
import { allOccurrencesBody, OCCURRENCES, period } from "./fixtures";
import { RoutineList } from "./RoutineList";

afterEach(() => vi.unstubAllGlobals());

describe("RoutineList", () => {
  it("shows every Routine from one request", async () => {
    const fetchMock = stubApi({ [OCCURRENCES]: () => jsonResponse(200, allOccurrencesBody) });
    renderWithClient(<RoutineList period={period} onEdit={() => undefined} />);
    expect(await screen.findByText("Office day")).toBeInTheDocument();
    expect(screen.getByText("Evening")).toBeInTheDocument();
    const asked = fetchMock.mock.calls.map((call) => new URL(call[0].url).pathname).filter((path) => path.startsWith("/api/time/routines"));
    expect(asked).toEqual(["/api/time/routines/occurrences"]);
  });

  it("says so when there are no Routines yet, and when loading fails", async () => {
    stubApi({ [OCCURRENCES]: () => jsonResponse(200, { routines: [] }) });
    const { unmount } = renderWithClient(<RoutineList period={period} onEdit={() => undefined} />);
    expect(await screen.findByText(/No Routines yet/)).toBeInTheDocument();
    unmount();
    stubApi({ [OCCURRENCES]: () => jsonResponse(500, { detail: "boom" }) });
    renderWithClient(<RoutineList period={period} onEdit={() => undefined} />);
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });
});
