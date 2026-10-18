import { render, screen } from "@testing-library/react";
import { FocusCard } from "./FocusCard";
import { summary } from "./testSummary";

const { overview } = summary();

describe("FocusCard", () => {
  it("labels the first and last Active moment of a Day", () => {
    render(<FocusCard overview={overview} clock="h24" isDay />);
    expect(screen.getByText("First Active")).toBeInTheDocument();
    expect(screen.getByText("Last Active")).toBeInTheDocument();
  });

  it("calls them earliest and latest for a longer Period", () => {
    render(<FocusCard overview={overview} clock="h24" isDay={false} />);
    expect(screen.getByText("Earliest Active")).toBeInTheDocument();
    expect(screen.getByText("Latest Active")).toBeInTheDocument();
    expect(screen.queryByText("First Active")).not.toBeInTheDocument();
  });

  it("shows the Focus figures", () => {
    render(<FocusCard overview={overview} clock="h24" isDay />);
    expect(screen.getByText("Longest Focus stretch")).toBeInTheDocument();
    expect(screen.getByText("45m")).toBeInTheDocument();
    expect(screen.getByText("40%")).toBeInTheDocument();
  });
});
