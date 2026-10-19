import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ClearFiltersButton } from "./ClearFiltersButton";

describe("ClearFiltersButton", () => {
  it("calls onClear when pressed", async () => {
    const onClear = vi.fn();
    render(<ClearFiltersButton onClear={onClear} />);
    await userEvent.click(screen.getByRole("button", { name: "Clear filters" }));
    expect(onClear).toHaveBeenCalledTimes(1);
  });
});
