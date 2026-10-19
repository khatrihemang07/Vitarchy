import { render, screen } from "@testing-library/react";
import { TooltipProvider } from "../ui/tooltip";
import { ChangeBadge } from "./ChangeBadge";

const show = (delta: number | null, text: string) =>
  render(
    <TooltipProvider>
      <ChangeBadge delta={delta} text={text} against="last month" />
    </TooltipProvider>,
  );

describe("ChangeBadge", () => {
  it("says which way the figure moved, in words for screen readers", () => {
    show(-500, "↓ ₹5.00");
    const badge = screen.getByText("↓ ₹5.00");
    expect(badge).toHaveAttribute("data-direction", "down");
    expect(badge).toHaveAttribute("aria-label", "down ₹5.00 from last month");
  });

  it("says no change plainly", () => {
    show(0, "—");
    expect(screen.getByText("—")).toHaveAttribute("aria-label", "No change from last month");
  });

  it("shows nothing when there is nothing to compare with", () => {
    const { container } = show(null, "");
    expect(container).toBeEmptyDOMElement();
  });
});
