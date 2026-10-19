import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ClearFiltersButton } from "./ClearFiltersButton";

describe("ClearFiltersButton", () => {
  it.each(["bar", "notice"] as const)("calls onClear when pressed in the %s", async (placement) => {
    const onClear = vi.fn();
    render(<ClearFiltersButton placement={placement} onClear={onClear} />);
    await userEvent.click(screen.getByRole("button", { name: "Clear filters" }));
    expect(onClear).toHaveBeenCalledTimes(1);
  });

  it("is a small ghost button in a filter bar and an outlined default one in a notice", () => {
    const { rerender } = render(<ClearFiltersButton placement="bar" onClear={() => undefined} />);
    expect(screen.getByRole("button")).toHaveAttribute("data-variant", "ghost");
    expect(screen.getByRole("button")).toHaveAttribute("data-size", "sm");
    rerender(<ClearFiltersButton placement="notice" onClear={() => undefined} />);
    expect(screen.getByRole("button")).toHaveAttribute("data-variant", "outline");
    expect(screen.getByRole("button")).toHaveAttribute("data-size", "default");
  });
});
