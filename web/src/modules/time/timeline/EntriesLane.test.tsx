import { render, screen } from "@testing-library/react";
import { TooltipProvider } from "@/design";
import { EntriesLane } from "./EntriesLane";

const entry = (from: number, to: number, description = "Build", project: string | null = "Office") => ({
  start_at: new Date(2026, 9, 12, from).toISOString(),
  stop_at: new Date(2026, 9, 12, to).toISOString(),
  description,
  project,
});

function renderLane(entries = [entry(6, 12)]) {
  return render(<TooltipProvider><EntriesLane day="2026-10-12" entries={entries} clock="h24" /></TooltipProvider>);
}

describe("EntriesLane", () => {
  it("counts the Day's Time Entries for screen readers and has a button for each", () => {
    renderLane([entry(6, 12), entry(13, 14, "")]);
    expect(screen.getByRole("group", { name: "Time Entries on this Day: 2" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "06:00 – 12:00, Build, Office" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /\(no description\)/ })).toBeInTheDocument();
  });

  it("places an entry by its share of the Day", () => {
    renderLane();
    expect(screen.getByRole("button", { name: /Build/ })).toHaveStyle({ left: "25%", width: "25%" });
  });

  it("is an empty lane without entries", () => {
    renderLane([]);
    expect(screen.getByRole("group", { name: "Time Entries on this Day: 0" })).toBeInTheDocument();
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });
});
