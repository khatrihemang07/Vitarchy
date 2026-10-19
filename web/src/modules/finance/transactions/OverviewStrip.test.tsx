import { render, screen, within } from "@testing-library/react";
import { TooltipProvider } from "@/design";
import { overview } from "../test-utils";
import { OverviewStrip } from "./OverviewStrip";

const show = (value = overview()) =>
  render(
    <TooltipProvider>
      <OverviewStrip overview={value} />
    </TooltipProvider>,
  );

describe("OverviewStrip", () => {
  it("shows the four figures with how each moved against the period just before", () => {
    show();
    const strip = screen.getByRole("region", { name: "Overview" });
    expect(within(strip).getAllByText("₹5,000.00").length).toBeGreaterThan(0);
    expect(within(strip).getByText("↓ ₹1,500.00")).toHaveAttribute("data-direction", "down");
    expect(within(strip).getByText("—")).toHaveAttribute("aria-label", "No change from the period just before");
  });

  it("says Transfers are left out of Money in and Money out", () => {
    show();
    expect(screen.getByText("Transfers between your Accounts are left out of Money in and Money out.")).toBeInTheDocument();
  });

  it("gives screen readers each day's Money in and Money out as text", () => {
    show();
    const row = screen.getByRole("row", { name: /Wed 1 Apr/ });
    expect(within(row).getAllByRole("cell").map((c) => c.textContent)).toEqual(["₹5,000.00", "₹0.00"]);
  });

  it("leaves the chart out for a single day", () => {
    show(overview({ daily: [{ date: "2026-04-01", money_in_paise: 1, money_out_paise: 0 }] }));
    expect(screen.queryByRole("figure")).not.toBeInTheDocument();
  });
});
