import { render, screen, within } from "@testing-library/react";
import { DayTimeline } from "./DayTimeline";

const props = {
  dayStart: "2026-05-01T00:00:00Z",
  dayEnd: "2026-05-02T00:00:00Z",
  tz: "UTC",
  clock: "h24" as const,
  subject: "Activity",
  formatSeconds: (s: number) => `${String(s / 60)} min`,
};

describe("DayTimeline", () => {
  it("names each label in the legend and lists every stretch in a table", () => {
    render(
      <DayTimeline
        {...props}
        segments={[
          { start: "2026-05-01T09:00:00Z", end: "2026-05-01T10:00:00Z", label: "vitarchy", detail: "Code" },
          { start: "2026-05-01T12:00:00Z", end: "2026-05-01T12:30:00Z", label: "YouTube" },
        ]}
      />,
    );
    expect(screen.getAllByRole("listitem").map((item) => item.textContent)).toEqual(["vitarchy", "YouTube"]);
    const rows = within(screen.getByRole("table", { hidden: true })).getAllByRole("row", { hidden: true });
    expect(rows).toHaveLength(3);
    expect(rows[1]).toHaveTextContent("09:00 – 10:00");
    expect(rows[1]).toHaveTextContent("vitarchy (Code)");
    expect(rows[2]).toHaveTextContent("30 min");
  });

  it("describes the lane for assistive technology", () => {
    render(<DayTimeline {...props} segments={[{ start: "2026-05-01T09:00:00Z", end: "2026-05-01T10:00:00Z", label: "A" }]} />);
    expect(screen.getByRole("img", { name: /Activity across the day, 1 stretches/ })).toBeInTheDocument();
  });

  it("shows one legend entry per group, in the group's own colour", () => {
    render(
      <DayTimeline
        {...props}
        segments={[
          { start: "2026-05-01T09:00:00Z", end: "2026-05-01T10:00:00Z", label: "job", group: "office", color: "#2a78d6" },
          { start: "2026-05-01T10:00:00Z", end: "2026-05-01T11:00:00Z", label: "travel", group: "office", color: "#2a78d6" },
        ]}
      />,
    );
    expect(screen.getAllByRole("listitem").map((item) => item.textContent)).toEqual(["office"]);
    expect(document.querySelector("rect")).toHaveAttribute("fill", "#2a78d6");
  });
});
