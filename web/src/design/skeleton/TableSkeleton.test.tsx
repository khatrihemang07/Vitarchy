import { render, screen } from "@testing-library/react";
import { TableSkeleton } from "./TableSkeleton";

describe("TableSkeleton", () => {
  it("announces what is loading as a busy status", () => {
    render(<TableSkeleton label="Loading Transactions" />);
    expect(screen.getByRole("status")).toHaveAttribute("aria-busy", "true");
    expect(screen.getByRole("status")).toHaveTextContent("Loading Transactions");
  });

  it("draws the requested rows", () => {
    render(<TableSkeleton label="Loading" rows={3} />);
    expect(screen.getAllByRole("listitem")).toHaveLength(3);
  });
});
