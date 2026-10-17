import { render, screen } from "@testing-library/react";
import { TooltipProvider } from "@/design";
import { formatSpan } from "../format";
import { OverviewStrip } from "./OverviewStrip";
import { group, report } from "./testReport";

describe("OverviewStrip", () => {
  it("gives the headline figures as text at once, whatever the counting animation does", () => {
    const data = report([group("Standup")]);
    render(<TooltipProvider><OverviewStrip report={data} /></TooltipProvider>);
    expect(screen.getByRole("heading", { name: "Overview" })).toBeInTheDocument();
    expect(screen.getByText(formatSpan(7200), { selector: ".sr-only" })).toBeInTheDocument();
    expect(screen.getByText(formatSpan(3600), { selector: ".sr-only" })).toBeInTheDocument();
    for (const title of ["Total", "Days tracked", "Average per tracked Day"]) {
      expect(screen.getByText(title)).toBeInTheDocument();
    }
  });
});
