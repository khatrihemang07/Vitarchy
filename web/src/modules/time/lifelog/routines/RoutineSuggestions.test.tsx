import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { jsonResponse, renderWithClient, stubApi } from "../../test-utils";
import { CREATE, OFFICE_SUGGESTION, period, savedRoutine, SUGGESTIONS } from "./fixtures";
import { RoutineSuggestions } from "./RoutineSuggestions";

afterEach(() => {
  vi.unstubAllGlobals();
  sessionStorage.clear();
});

describe("RoutineSuggestions", () => {
  it("shows the chain and confirms it with the name the person typed", async () => {
    let sent: unknown;
    stubApi({
      [SUGGESTIONS]: () => jsonResponse(200, { suggestions: [OFFICE_SUGGESTION] }),
      [CREATE]: async (request) => {
        sent = await request.json();
        return jsonResponse(201, savedRoutine);
      },
    });
    renderWithClient(<RoutineSuggestions period={period} />);
    expect(await screen.findByText("inside_office")).toBeInTheDocument();
    expect(screen.getByText(/on 8 days/)).toBeInTheDocument();
    const name = screen.getByLabelText("Name");
    await userEvent.clear(name);
    await userEvent.type(name, "Office day");
    await userEvent.click(screen.getByRole("button", { name: "Confirm" }));
    await waitFor(() => { expect(sent).toBeDefined(); });
    expect(sent).toEqual({
      name: "Office day",
      steps: OFFICE_SUGGESTION.steps.map((value) => ({ label: value, kind: "description", value })),
    });
  });

  it("can confirm a suggestion whose descriptions are long, labelling each Step within the limit", async () => {
    let sent: { steps: { label: string; value: string }[] } | undefined;
    const long = "a very long description ".repeat(6).trim();
    stubApi({
      [SUGGESTIONS]: () => jsonResponse(200, { suggestions: [{ steps: [long, "b"], days: 6 }] }),
      [CREATE]: async (request) => {
        sent = (await request.json()) as typeof sent;
        return jsonResponse(201, savedRoutine);
      },
    });
    renderWithClient(<RoutineSuggestions period={period} />);
    await userEvent.click(await screen.findByRole("button", { name: "Confirm" }));
    await waitFor(() => { expect(sent).toBeDefined(); });
    expect(Array.from(sent?.steps[0]?.label ?? "").length).toBeLessThanOrEqual(80);
    expect(sent?.steps[0]?.value).toBe(long);
  });

  it("hides a dismissed suggestion", async () => {
    stubApi({ [SUGGESTIONS]: () => jsonResponse(200, { suggestions: [OFFICE_SUGGESTION] }) });
    renderWithClient(<RoutineSuggestions period={period} />);
    await userEvent.click(await screen.findByRole("button", { name: "Dismiss" }));
    expect(await screen.findByText(/No new pattern/)).toBeInTheDocument();
  });

  it("explains a rejected confirmation", async () => {
    stubApi({
      [SUGGESTIONS]: () => jsonResponse(200, { suggestions: [OFFICE_SUGGESTION] }),
      [CREATE]: () => jsonResponse(409, { detail: "another Routine already has this name", code: "routine_name_taken" }),
    });
    renderWithClient(<RoutineSuggestions period={period} />);
    await userEvent.click(await screen.findByRole("button", { name: "Confirm" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("already has this name");
  });

  it("says so when nothing stands out, and when loading fails", async () => {
    stubApi({ [SUGGESTIONS]: () => jsonResponse(200, { suggestions: [] }) });
    const { unmount } = renderWithClient(<RoutineSuggestions period={period} />);
    expect(await screen.findByText(/No new pattern/)).toBeInTheDocument();
    unmount();
    stubApi({ [SUGGESTIONS]: () => jsonResponse(400, { detail: "bad", code: "unknown_time_zone" }) });
    renderWithClient(<RoutineSuggestions period={period} />);
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });
});
