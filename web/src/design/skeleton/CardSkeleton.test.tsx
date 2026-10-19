import { render, screen } from "@testing-library/react";
import { CardSkeleton } from "./CardSkeleton";

describe("CardSkeleton", () => {
  it("announces what is loading as a busy status", () => {
    render(<CardSkeleton label="Loading Accounts" />);
    expect(screen.getByRole("status")).toHaveAttribute("aria-busy", "true");
    expect(screen.getByRole("status")).toHaveTextContent("Loading Accounts");
  });
});
