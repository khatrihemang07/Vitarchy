import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { TooltipProvider } from "../ui/tooltip";
import { SortButton } from "./SortButton";

const show = (order: "asc" | "desc" | null, onSort = vi.fn()) =>
  render(
    <TooltipProvider>
      <SortButton label="Date" tooltip="When it happened" sort={{ order, onSort }} />
    </TooltipProvider>,
  );

describe("SortButton", () => {
  it.each([
    ["asc", "ascending"],
    ["desc", "descending"],
    [null, "none"],
  ] as const)("says it is %s to assistive tech", (order, expected) => {
    show(order);
    expect(screen.getByRole("columnheader")).toHaveAttribute("aria-sort", expected);
  });

  it("asks to sort when clicked", async () => {
    const onSort = vi.fn();
    show(null, onSort);
    await userEvent.click(screen.getByRole("button", { name: "Date" }));
    expect(onSort).toHaveBeenCalledOnce();
  });
});
