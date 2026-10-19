import { render, screen } from "@testing-library/react";
import { TooltipProvider } from "../ui/tooltip";
import { ColumnHeaderBar } from "./ColumnHeaderBar";

describe("ColumnHeaderBar", () => {
  it("shows every heading, making only the ones with an explanation focusable", () => {
    render(
      <TooltipProvider>
        <ColumnHeaderBar
          grid="md:grid"
          columns={[
            { id: "name", label: "Name" },
            { id: "amount", label: "Amount", tooltip: "What moved", align: "end" },
          ]}
        />
      </TooltipProvider>,
    );
    expect(screen.getByText("Name")).not.toHaveAttribute("tabindex");
    expect(screen.getByText("Amount")).toHaveAttribute("tabindex", "0");
    expect(screen.getByText("Amount")).toHaveClass("justify-self-end");
  });
});
