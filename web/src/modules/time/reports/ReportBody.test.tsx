import { render } from "@testing-library/react";
import { TooltipProvider } from "@/design";
import { ReportBody } from "./ReportBody";
import { group, report } from "./testReport";

function delays(container: HTMLElement): string[] {
  return [...container.querySelectorAll<HTMLElement>(".animate-fade-up")].map((el) => el.style.animationDelay);
}

function renderBody(kind: "week" | "day") {
  return render(
    <TooltipProvider>
      <ReportBody report={report([group("Standup")], kind)} mode="per_day" compare="previous" clock="h24" filtered={false} onClear={() => undefined} firstStagger={3} />
    </TooltipProvider>,
  );
}

describe("ReportBody", () => {
  it("fades its sections in one after another, continuing after the sections above it", () => {
    const { container } = renderBody("week");
    expect(delays(container)).toEqual(["210ms", "280ms"]);
  });

  it("staggers the entries section too for a one-Day Report", () => {
    const { container } = renderBody("day");
    expect(delays(container)).toEqual(["210ms", "280ms", "350ms"]);
  });
});
