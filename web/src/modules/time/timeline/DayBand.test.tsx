import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { TooltipProvider } from "@/design";
import { appColorMap } from "./appColors";
import { DayBand } from "./DayBand";
import { summary } from "./testSummary";

const data = summary({}, "day");
const base = data.segments ?? [];
const at = (i: number) => base[i] as (typeof base)[number];
const segments = [
  { ...at(0), start_at: new Date(2026, 9, 12, 8).toISOString(), end_at: new Date(2026, 9, 12, 9).toISOString() },
  { ...at(1), start_at: new Date(2026, 9, 12, 9).toISOString(), end_at: new Date(2026, 9, 12, 9, 30).toISOString() },
  { ...at(2), start_at: new Date(2026, 9, 12, 9, 30).toISOString(), end_at: new Date(2026, 9, 12, 10).toISOString() },
];

function renderBand(highlight?: string, now = new Date(2026, 9, 13, 12)) {
  return render(
    <TooltipProvider>
      <DayBand day="2026-10-12" segments={segments} colors={appColorMap(data.apps)} clock="h24" highlight={highlight} now={now} summary="Timeline of Mon 12 Oct: 5h Active" />
    </TooltipProvider>,
  );
}

describe("DayBand", () => {
  it("is summarised for screen readers and has a button for each Segment", () => {
    renderBand();
    expect(screen.getByRole("toolbar", { name: "Timeline of Mon 12 Oct: 5h Active" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "08:00 – 09:00 · 1h, Code, main.rs, Active" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "09:30 – 10:00 · 30m, (no app), Away" })).toBeInTheDocument();
  });

  it("describes a Segment in a tooltip on focus", async () => {
    renderBand();
    await userEvent.tab();
    expect((await screen.findAllByText("main.rs")).length).toBeGreaterThan(0);
  });

  it("is one Tab stop, and the arrow keys move between Segments", async () => {
    renderBand();
    await userEvent.tab();
    expect(screen.getByRole("button", { name: /Code, main.rs/ })).toHaveFocus();
    await userEvent.keyboard("{ArrowRight}");
    expect(screen.getByRole("button", { name: /Safari, Docs/ })).toHaveFocus();
    await userEvent.keyboard("{End}");
    expect(screen.getByRole("button", { name: /\(no app\), Away/ })).toHaveFocus();
    await userEvent.keyboard("{Home}");
    expect(screen.getByRole("button", { name: /Code, main.rs/ })).toHaveFocus();
  });

  it("dims the other apps when one is highlighted", () => {
    renderBand("Code");
    expect(screen.getByRole("button", { name: /Safari, Docs/ })).toHaveStyle({ opacity: "0.25" });
    expect(screen.getByRole("button", { name: /Code, main.rs/ })).toHaveStyle({ opacity: "1" });
  });

  it("marks now only on the Day it is", () => {
    const { container, unmount } = renderBand(undefined, new Date(2026, 9, 12, 12));
    expect(container.querySelector('[title="Now"]')).not.toBeNull();
    unmount();
    expect(renderBand().container.querySelector('[title="Now"]')).toBeNull();
  });
});
