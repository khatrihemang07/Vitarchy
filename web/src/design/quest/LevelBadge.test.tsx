import { render, screen } from "@testing-library/react";
import { LevelBadge } from "./LevelBadge";

describe("LevelBadge", () => {
  it("labels the level for assistive tech", () => {
    render(<LevelBadge level={12} />);
    expect(screen.getByLabelText("Level 12")).toHaveTextContent("12");
  });

  it("supports level 0", () => {
    render(<LevelBadge level={0} />);
    expect(screen.getByLabelText("Level 0")).toHaveTextContent("0");
  });

  it("floors fractions and clamps negatives to 0", () => {
    render(<LevelBadge level={-2.7} />);
    expect(screen.getByLabelText("Level 0")).toBeInTheDocument();
  });
});
