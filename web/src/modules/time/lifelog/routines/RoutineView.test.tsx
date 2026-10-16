import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { jsonResponse, renderWithClient, stubApi } from "../../test-utils";
import { occurrencesBody, period, savedRoutine } from "./fixtures";
import { RoutineView } from "./RoutineView";

const OCCURRENCES = "GET /api/time/routines/3/occurrences";

afterEach(() => vi.unstubAllGlobals());

describe("RoutineView", () => {
  it("leads with the door-to-door time, lists each Step and the incomplete Days", async () => {
    stubApi({ [OCCURRENCES]: () => jsonResponse(200, occurrencesBody) });
    renderWithClient(<RoutineView routine={savedRoutine as never} period={period} onEdit={() => undefined} />);
    expect(await screen.findByText("9 h 15 min")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "To office" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Incomplete days" })).toBeInTheDocument();
    expect(screen.getByRole("cell", { name: "To office" })).toBeInTheDocument();
    expect(screen.getByText("View as table")).toBeInTheDocument();
  });

  it("says when the Routine did not happen, and when loading fails", async () => {
    stubApi({ [OCCURRENCES]: () => jsonResponse(200, { ...occurrencesBody, occurrences: [] }) });
    const { unmount } = renderWithClient(<RoutineView routine={savedRoutine as never} period={period} onEdit={() => undefined} />);
    expect(await screen.findByText(/did not happen/)).toBeInTheDocument();
    unmount();
    stubApi({ [OCCURRENCES]: () => jsonResponse(404, { detail: "no such", code: "routine_not_found" }) });
    renderWithClient(<RoutineView routine={savedRoutine as never} period={period} onEdit={() => undefined} />);
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });

  it("asks before deleting, then deletes", async () => {
    const del = vi.fn(() => new Response(null, { status: 204 }));
    stubApi({ [OCCURRENCES]: () => jsonResponse(200, occurrencesBody), "DELETE /api/time/routines/3": del });
    renderWithClient(<RoutineView routine={savedRoutine as never} period={period} onEdit={() => undefined} />);
    await userEvent.click(await screen.findByRole("button", { name: /Delete/ }));
    expect(del).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole("button", { name: "Delete Office day?" }));
    await waitFor(() => { expect(del).toHaveBeenCalledOnce(); });
  });

  it("opens the editor from Edit", async () => {
    const onEdit = vi.fn();
    stubApi({ [OCCURRENCES]: () => jsonResponse(200, occurrencesBody) });
    renderWithClient(<RoutineView routine={savedRoutine as never} period={period} onEdit={onEdit} />);
    await userEvent.click(await screen.findByRole("button", { name: /Edit/ }));
    expect(onEdit).toHaveBeenCalledOnce();
  });
});
