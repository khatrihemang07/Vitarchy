import { render, screen } from "@testing-library/react";
import { MotionProvider, TooltipProvider } from "@/design";
import { formatSpan } from "../format";
import { OverviewStrip } from "./OverviewStrip";
import { group, report } from "./testReport";

const data = report([group("Standup")]);

const renderStrip = (compare: "previous" | "usual", rows = data) =>
  render(<MotionProvider><TooltipProvider><OverviewStrip report={rows} compare={compare} /></TooltipProvider></MotionProvider>);

describe("OverviewStrip", () => {
  it("shows the headline figures as plain text at once", () => {
    renderStrip("previous");
    expect(screen.getByRole("heading", { name: "Overview" })).toBeInTheDocument();
    expect(screen.getAllByText(formatSpan(7200)).length).toBeGreaterThan(0);
    expect(screen.getAllByText(formatSpan(3600)).length).toBeGreaterThan(0);
    for (const title of ["Total", "Days tracked", "Average per tracked Day"]) expect(screen.getByText(title)).toBeInTheDocument();
  });

  it("says what the changes are measured against, and uses that comparison's changes", () => {
    const { unmount } = renderStrip("previous");
    expect(screen.getByText("Compared with the Previous Period, 5–11 Oct 2026")).toBeInTheDocument();
    expect(screen.getByText("↑ 10m")).toBeInTheDocument();
    unmount();
    renderStrip("usual", { ...data, overview: { ...data.overview, vs_usual: { ...data.overview.vs_usual, total_s: -1800 } } });
    expect(screen.getByText(/^Compared with the last 4 weeks, 14 Sep – 11 Oct 2026/)).toBeInTheDocument();
    expect(screen.getByText("↓ 30m")).toBeInTheDocument();
  });
});
