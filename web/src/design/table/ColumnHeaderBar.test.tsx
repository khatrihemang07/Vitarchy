import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
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

  it("turns a heading into a sort button when it has a sort", async () => {
    const onSort = vi.fn();
    render(
      <TooltipProvider>
        <ColumnHeaderBar grid="md:grid" columns={[{ id: "date", label: "Date", sort: { order: "desc", onSort } }, { id: "name", label: "Name" }]} />
      </TooltipProvider>,
    );
    expect(screen.getByRole("columnheader")).toHaveAttribute("aria-sort", "descending");
    await userEvent.click(screen.getByRole("button", { name: "Date" }));
    expect(onSort).toHaveBeenCalledOnce();
    expect(screen.queryByRole("button", { name: "Name" })).not.toBeInTheDocument();
  });
});
