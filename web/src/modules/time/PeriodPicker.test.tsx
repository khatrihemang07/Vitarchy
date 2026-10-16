import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { PeriodPicker } from "./PeriodPicker";
import type { PeriodSelection } from "./period";

const week: PeriodSelection = { kind: "week", from: "2026-10-05", to: "2026-10-11" };

function setup(value = week, today = "2026-10-14") {
  const onChange = vi.fn();
  render(<PeriodPicker value={value} weekStart="monday" today={today} onChange={onChange} />);
  return onChange;
}

describe("PeriodPicker", () => {
  it("names the period being looked at", () => {
    setup();
    expect(screen.getByText(/Oct 5.*11, 2026/)).toBeInTheDocument();
  });

  it("moves to the previous and next period", async () => {
    const onChange = setup();
    await userEvent.click(screen.getByRole("button", { name: "Previous period" }));
    expect(onChange).toHaveBeenLastCalledWith({ kind: "week", from: "2026-09-28", to: "2026-10-04" });
    await userEvent.click(screen.getByRole("button", { name: "Next period" }));
    expect(onChange).toHaveBeenLastCalledWith({ kind: "week", from: "2026-10-12", to: "2026-10-18" });
  });

  it("cannot move past the current period", () => {
    setup({ kind: "week", from: "2026-10-12", to: "2026-10-18" });
    expect(screen.getByRole("button", { name: "Next period" })).toBeDisabled();
  });

  it("switches kind, keeping where the person was", async () => {
    const onChange = setup();
    await userEvent.click(screen.getByRole("radio", { name: "Month" }));
    expect(onChange).toHaveBeenCalledWith({ kind: "month", from: "2026-10-01", to: "2026-10-31" });
  });

  it("shows date fields only for a custom range", () => {
    setup({ kind: "custom", from: "2026-10-01", to: "2026-10-09" });
    expect(screen.getByLabelText("From")).toHaveValue("2026-10-01");
    expect(screen.getByLabelText("To")).toHaveValue("2026-10-09");
  });
});
