import { render, screen } from "@testing-library/react";
import { XpBar } from "./XpBar";

describe("XpBar", () => {
  it("exposes progress to assistive tech", () => {
    render(<XpBar value={2340} max={3000} label="Level 12 progress" />);
    const bar = screen.getByRole("progressbar", { name: "Level 12 progress" });
    expect(bar).toHaveAttribute("aria-valuemin", "0");
    expect(bar).toHaveAttribute("aria-valuemax", "3000");
    expect(bar).toHaveAttribute("aria-valuenow", "2340");
    expect(bar).toHaveAttribute("aria-valuetext", "2340 of 3000 XP");
  });

  it("clamps the exposed value", () => {
    render(<XpBar value={9000} max={3000} label="Overflow" />);
    expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "3000");
  });

  it("renders an empty bar for a zero max", () => {
    render(<XpBar value={5} max={0} label="Empty" />);
    expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "0");
  });
});
