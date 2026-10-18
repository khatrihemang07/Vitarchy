import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { SegmentView } from "@/api/time";
import { appColorMap } from "./appColors";
import { INITIAL_LIMIT, SegmentList } from "./SegmentList";
import { summary } from "./testSummary";

const data = summary({}, "day");
const colors = appColorMap(data.apps);
const segments = data.segments ?? [];

const many = (n: number): SegmentView[] =>
  Array.from({ length: n }, (_, i) => ({ start_at: new Date(2026, 9, 12, 0, i).toISOString(), end_at: new Date(2026, 9, 12, 0, i + 1).toISOString(), app: "Code", title: `File ${String(i)}`, url: null, away: "no", sources: ["toggl"] }));

describe("SegmentList", () => {
  it("lists time, app, title, site, length and Away badges", () => {
    render(<SegmentList segments={segments} colors={colors} clock="h24" />);
    expect(screen.getByText("main.rs")).toBeInTheDocument();
    expect(screen.getByText("Docs · docs.rs")).toBeInTheDocument();
    expect(screen.getByText("Away", { selector: "[data-slot=badge]" })).toBeInTheDocument();
  });

  it("shows only the highlighted app's Segments", () => {
    render(<SegmentList segments={segments} colors={colors} clock="h24" highlight="Safari" />);
    expect(screen.getByText("Docs · docs.rs")).toBeInTheDocument();
    expect(screen.queryByText("main.rs")).not.toBeInTheDocument();
  });

  it("says so when the highlighted app has no Segments", () => {
    render(<SegmentList segments={segments} colors={colors} clock="h24" highlight="Figma" />);
    expect(screen.getByText("No Segments in Figma on this Day.")).toBeInTheDocument();
  });

  it("shows the first 50 and then all on request", async () => {
    render(<SegmentList segments={many(INITIAL_LIMIT + 10)} colors={colors} clock="h24" />);
    expect(screen.getAllByRole("listitem")).toHaveLength(INITIAL_LIMIT);
    await userEvent.click(screen.getByRole("button", { name: "Show all 60 Segments" }));
    expect(screen.getAllByRole("listitem")).toHaveLength(60);
    expect(screen.queryByRole("button", { name: /Show all/ })).not.toBeInTheDocument();
  });
});
