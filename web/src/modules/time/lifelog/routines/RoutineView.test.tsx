import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { renderWithClient, stubApi } from "../../test-utils";
import { occurrencesBody } from "./fixtures";
import { RoutineView } from "./RoutineView";

afterEach(() => vi.unstubAllGlobals());

const show = (entry: object, onEdit = () => undefined) => renderWithClient(<RoutineView entry={entry as never} onEdit={onEdit} />);

describe("RoutineView", () => {
  it("leads with the door-to-door time, lists each Step and the incomplete Days", () => {
    show(occurrencesBody);
    expect(screen.getByText("9 h 15 min")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "To office" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Incomplete days" })).toBeInTheDocument();
    expect(screen.getByRole("cell", { name: "To office" })).toBeInTheDocument();
    expect(screen.getByText("View as table")).toBeInTheDocument();
  });

  it("names Steps from the same response as the figures, never from another list", () => {
    const renamed = { ...occurrencesBody, routine: { ...occurrencesBody.routine, steps: occurrencesBody.routine.steps.map((s) => ({ ...s, label: `${s.label} (new)` })) } };
    show(renamed);
    expect(screen.getByRole("heading", { name: "To office (new)" })).toBeInTheDocument();
  });

  it("says when the Routine did not happen", () => {
    show({ ...occurrencesBody, occurrences: [] });
    expect(screen.getByText(/did not happen/)).toBeInTheDocument();
  });

  it("asks before deleting, then deletes", async () => {
    const del = vi.fn(() => new Response(null, { status: 204 }));
    stubApi({ "DELETE /api/time/routines/3": del });
    show(occurrencesBody);
    await userEvent.click(screen.getByRole("button", { name: /Delete/ }));
    expect(del).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole("button", { name: "Delete Office day?" }));
    await waitFor(() => { expect(del).toHaveBeenCalledOnce(); });
  });

  it("opens the editor from Edit", async () => {
    const onEdit = vi.fn();
    show(occurrencesBody, onEdit);
    await userEvent.click(screen.getByRole("button", { name: /Edit/ }));
    expect(onEdit).toHaveBeenCalledOnce();
  });
});
