import { render, screen } from "@testing-library/react";
import { StreakFlame } from "./StreakFlame";

describe("StreakFlame", () => {
  it("shows a cold state with no streak", () => {
    render(<StreakFlame days={0} />);
    expect(screen.getByText("No streak yet")).toBeInTheDocument();
    expect(screen.getByTestId("streak-flame")).toHaveAttribute("data-tier", "cold");
  });

  it("shows the day count and singular wording", () => {
    render(<StreakFlame days={1} />);
    expect(screen.getByText("1-day streak")).toBeInTheDocument();
    expect(screen.getByTestId("streak-flame")).toHaveAttribute("data-tier", "warm");
  });

  it("burns hot for a week or more", () => {
    render(<StreakFlame days={9} />);
    expect(screen.getByText("9-day streak")).toBeInTheDocument();
    expect(screen.getByTestId("streak-flame")).toHaveAttribute("data-tier", "hot");
  });
});
