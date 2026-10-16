import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { LIFELOG_OVERVIEW, lifeLogBody } from "../overviewFixture";
import { jsonResponse, renderWithClient, stubApi } from "../../test-utils";
import { CREATE, period, savedRoutine } from "./fixtures";
import { RoutineEditor } from "./RoutineEditor";

afterEach(() => vi.unstubAllGlobals());

const open = (routine?: typeof savedRoutine) =>
  renderWithClient(<RoutineEditor open onOpenChange={() => undefined} routine={routine as never} period={period} />);

describe("RoutineEditor", () => {
  it("cannot save until the name and every Step are filled in, then sends them in order", async () => {
    let sent: unknown;
    stubApi({
      [LIFELOG_OVERVIEW]: () => jsonResponse(200, lifeLogBody()),
      [CREATE]: async (request) => {
        sent = await request.json();
        return jsonResponse(201, savedRoutine);
      },
    });
    open();
    const save = screen.getByRole("button", { name: "Save" });
    expect(save).toBeDisabled();
    await userEvent.type(screen.getByLabelText("Name"), "Walk");
    await userEvent.type(screen.getByLabelText("Label"), "Out");
    await userEvent.type(screen.getByLabelText("Description"), "walk");
    expect(save).toBeEnabled();
    await userEvent.click(screen.getByRole("button", { name: "Add step" }));
    expect(save).toBeDisabled();
    await userEvent.type(screen.getAllByLabelText("Label")[1] as HTMLElement, "Back");
    await userEvent.type(screen.getAllByLabelText("Description")[1] as HTMLElement, "return");
    await userEvent.click(screen.getByRole("button", { name: "Move step 2 up" }));
    await userEvent.click(save);
    await waitFor(() => { expect(sent).toBeDefined(); });
    expect(sent).toEqual({
      name: "Walk",
      steps: [
        { label: "Back", kind: "description", value: "return" },
        { label: "Out", kind: "description", value: "walk" },
      ],
    });
  });

  it("removes a Step and edits an existing Routine with a PUT", async () => {
    const put = vi.fn(() => jsonResponse(200, savedRoutine));
    stubApi({ [LIFELOG_OVERVIEW]: () => jsonResponse(200, lifeLogBody()), "PUT /api/time/routines/3": put });
    open(savedRoutine);
    expect(screen.getByDisplayValue("Office day")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Remove step 2" }));
    await userEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => { expect(put).toHaveBeenCalledOnce(); });
  });

  it("shows the server's reason when saving is refused", async () => {
    stubApi({
      [LIFELOG_OVERVIEW]: () => jsonResponse(200, lifeLogBody()),
      "PUT /api/time/routines/3": () => jsonResponse(409, { detail: "another Routine already has this name", code: "routine_name_taken" }),
    });
    open(savedRoutine);
    await userEvent.click(screen.getByRole("button", { name: "Save" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("already has this name");
  });

  it("cannot add a ninth Step", async () => {
    stubApi({ [LIFELOG_OVERVIEW]: () => jsonResponse(200, lifeLogBody()) });
    open();
    const add = screen.getByRole("button", { name: "Add step" });
    for (let i = 0; i < 7; i += 1) await userEvent.click(add);
    expect(add).toBeDisabled();
  });
});
