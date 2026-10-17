import type { ReactElement } from "react";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { TooltipProvider } from "@/design";
import { GroupRow } from "./GroupRow";
import { type RowContext, rowContextFor } from "./rowContext";
import { group, report } from "./testReport";

const contextFor = (kind: "week" | "day" = "week", compare: "previous" | "usual" = "previous"): RowContext =>
  rowContextFor({ period: report([], kind).period, mode: "per_day", compare, clock: "h24" });
const context = contextFor();

const renderRow = (ui: ReactElement) => render(<TooltipProvider>{ui}</TooltipProvider>);

describe("GroupRow", () => {
  it("starts collapsed and opens its entries when pressed", async () => {
    renderRow(<ul><GroupRow group={group("Standup")} context={context} /></ul>);
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
    renderRow(<ul><GroupRow group={group("Standup")} context={context} /></ul>);
    await userEvent.tab();
    await userEvent.keyboard("{Enter}");
    expect(screen.getByRole("button", { name: /Standup/ })).toHaveAttribute("aria-expanded", "true");
  });

  it("shows its second-level rows instead of entries, all closed", async () => {
    const parent = group("Office", { children: [group("Review"), group("Plan")], entries: [] });
    renderRow(<ul><GroupRow group={parent} context={context} /></ul>);
    expect(screen.queryByRole("button", { name: /Review/ })).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: /Office/ }));
    expect(screen.getByRole("button", { name: /Review/ })).toHaveAttribute("aria-expanded", "false");
    expect(screen.getByRole("button", { name: /Plan/ })).toHaveAttribute("aria-expanded", "false");
    expect(screen.queryByText("Review work")).not.toBeInTheDocument();
  });

  it("does not open a bottom-level row of a one-Day Report", () => {
    renderRow(<ul><GroupRow group={group("Standup")} context={contextFor("day")} /></ul>);
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
    expect(screen.getByText("Standup")).toBeInTheDocument();
    expect(screen.queryByText("Standup work")).not.toBeInTheDocument();
  });

  it("still opens a top-level row of a one-Day Report to show its second-level rows", async () => {
    const parent = group("Office", { children: [group("Review")], entries: [] });
    renderRow(<ul><GroupRow group={parent} context={contextFor("day")} /></ul>);
    await userEvent.click(screen.getByRole("button", { name: /Office/ }));
    expect(screen.getByText("Review")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Review/ })).not.toBeInTheDocument();
  });

  it("shows days out of the Period, total and its change, and marks a new group", () => {
    renderRow(<ul><GroupRow group={group("Standup", { vs_previous: { is_new: true, change: null }, vs_usual: { is_new: true } })} context={context} /></ul>);
    expect(screen.getByText("1 of 7")).toBeInTheDocument();
    expect(screen.getByText("new")).toBeInTheDocument();
  });

  it("shows the change from the Previous Period", () => {
    renderRow(<ul><GroupRow group={group("Standup")} context={context} /></ul>);
    expect(screen.getByText("↑ 15m")).toBeInTheDocument();
  });

  it("shows the change from the last 4 weeks, and 'new' when the group is missing from them", () => {
    const usualChange = { days: 0, entries: 0, total_s: -1800, per_day: { avg_s: 0, min_s: 0, max_s: 0 }, per_entry: { avg_s: 0, min_s: 0, max_s: 0 } };
    const seen = group("Standup", { vs_usual: { is_new: false, change: usualChange } });
    const { unmount } = renderRow(<ul><GroupRow group={seen} context={contextFor("week", "usual")} /></ul>);
    expect(screen.getByText("↓ 30m")).toBeInTheDocument();
    expect(screen.queryByText("new")).not.toBeInTheDocument();
    unmount();
    renderRow(<ul><GroupRow group={group("Standup", { vs_usual: { is_new: true, change: null } })} context={contextFor("week", "usual")} /></ul>);
    expect(screen.getByText("new")).toBeInTheDocument();
  });

  it("reserves no chevron space for a top-level row that cannot open", () => {
    renderRow(<ul><GroupRow group={group("Standup")} context={contextFor("day")} /></ul>);
    expect(screen.getByText("Standup").previousElementSibling).toBeNull();
  });

  it("lines a nested row that cannot open up under its parent's label", async () => {
    const parent = group("Office", { children: [group("Review")], entries: [] });
    renderRow(<ul><GroupRow group={parent} context={contextFor("day")} /></ul>);
    await userEvent.click(screen.getByRole("button", { name: /Office/ }));
    expect(screen.getByText("Review").previousElementSibling).not.toBeNull();
  });
});
