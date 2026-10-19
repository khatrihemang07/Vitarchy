import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { PeriodPicker } from "./PeriodPicker";

const handlers = () => ({ onKind: vi.fn(), onShift: vi.fn(), onToday: vi.fn(), onCustom: vi.fn() });
const week = { from: "2026-10-12", to: "2026-10-18" };

describe("PeriodPicker", () => {
  it("shows the Period's label and moves with the arrows", async () => {
    const h = handlers();
    render(<PeriodPicker kind="week" range={week} isCurrent={false} {...h} />);
    expect(screen.getByText("12–18 Oct 2026")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Previous week" }));
    await userEvent.click(screen.getByRole("button", { name: "Next week" }));
    expect(h.onShift.mock.calls).toEqual([[-1], [1]]);
  });

  it("changes kind and jumps to today, which is disabled while already there", async () => {
    const h = handlers();
    const { rerender } = render(<PeriodPicker kind="week" range={week} isCurrent={false} {...h} />);
    await userEvent.click(screen.getByRole("radio", { name: "Month" }));
    expect(h.onKind).toHaveBeenCalledWith("month");
    await userEvent.click(screen.getByRole("button", { name: "Today" }));
    expect(h.onToday).toHaveBeenCalled();
    rerender(<PeriodPicker kind="week" range={week} isCurrent {...h} />);
    expect(screen.getByRole("button", { name: "Today" })).toBeDisabled();
  });

  it("offers date inputs only for a custom stretch", () => {
    const h = handlers();
    const { rerender } = render(<PeriodPicker kind="week" range={week} isCurrent={false} {...h} />);
    expect(screen.queryByLabelText("From")).not.toBeInTheDocument();
    rerender(<PeriodPicker kind="custom" range={week} isCurrent={false} {...h} />);
    expect(screen.getByLabelText("From")).toHaveValue("2026-10-12");
    expect(screen.getByLabelText("To")).toHaveValue("2026-10-18");
  });
});
