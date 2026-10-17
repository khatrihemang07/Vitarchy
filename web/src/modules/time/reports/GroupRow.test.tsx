import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { GroupRow } from "./GroupRow";
import type { RowContext } from "./rowContext";
import { group } from "./testReport";

const context: RowContext = { mode: "per_day", clock: "h24", periodDays: 7, isDay: false };

describe("GroupRow", () => {
  it("starts collapsed and opens its entries when pressed", async () => {
    render(<ul><GroupRow group={group("Standup")} context={context} /></ul>);
    const row = screen.getByRole("button", { name: /Standup/ });
    expect(row).toHaveAttribute("aria-expanded", "false");
    expect(screen.queryByText("Standup work")).not.toBeInTheDocument();
    await userEvent.click(row);
    expect(row).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByText("Standup work")).toBeInTheDocument();
    await userEvent.click(row);
    expect(screen.queryByText("Standup work")).not.toBeInTheDocument();
  });

  it("works from the keyboard", async () => {
    render(<ul><GroupRow group={group("Standup")} context={context} /></ul>);
    await userEvent.tab();
    await userEvent.keyboard("{Enter}");
    expect(screen.getByRole("button", { name: /Standup/ })).toHaveAttribute("aria-expanded", "true");
  });

  it("shows its second-level rows instead of entries", () => {
    const parent = group("Office", { children: [group("Review")], entries: [] });
    render(<ul><GroupRow group={parent} context={context} defaultOpen /></ul>);
    expect(screen.getByRole("button", { name: /Review/ })).toBeInTheDocument();
  });

  it("opens the first child of a row that opens by default", () => {
    const parent = group("Office", { children: [group("Review"), group("Plan")], entries: [] });
    render(<ul><GroupRow group={parent} context={context} defaultOpen /></ul>);
    expect(screen.getByRole("button", { name: /Review/ })).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByRole("button", { name: /Plan/ })).toHaveAttribute("aria-expanded", "false");
    expect(screen.getByText("Review work")).toBeInTheDocument();
  });

  it("does not open a bottom-level row of a one-Day Report", () => {
    render(<ul><GroupRow group={group("Standup")} context={{ ...context, isDay: true }} defaultOpen /></ul>);
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
    expect(screen.getByText("Standup")).toBeInTheDocument();
    expect(screen.queryByText("Standup work")).not.toBeInTheDocument();
  });

  it("still opens a top-level row of a one-Day Report to show its second-level rows", async () => {
    const parent = group("Office", { children: [group("Review")], entries: [] });
    render(<ul><GroupRow group={parent} context={{ ...context, isDay: true }} /></ul>);
    await userEvent.click(screen.getByRole("button", { name: /Office/ }));
    expect(screen.getByText("Review")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Review/ })).not.toBeInTheDocument();
  });

  it("shows days out of the Period, total and its change, and marks a new group", () => {
    render(<ul><GroupRow group={group("Standup", { is_new: true, change: null })} context={context} /></ul>);
    expect(screen.getByText("1 of 7")).toBeInTheDocument();
    expect(screen.getByText("new")).toBeInTheDocument();
  });

  it("shows the change from the Previous Period", () => {
    render(<ul><GroupRow group={group("Standup")} context={context} /></ul>);
    expect(screen.getByText("↑ 15m")).toBeInTheDocument();
  });
});
