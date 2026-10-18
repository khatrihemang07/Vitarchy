import { render } from "@testing-library/react";
import { stagger, TooltipProvider } from "@/design";
import { ReportBody } from "./ReportBody";
import { group, report } from "./testReport";

/** The delays of sections starting at place 3, derived from `stagger` so a changed step needs no edit here. */
const expected = (count: number) => Array.from({ length: count }, (_, i) => stagger(3 + i).animationDelay);

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
    expect(delays(container)).toEqual(expected(2));
  });

  it("staggers the entries section too for a one-Day Report", () => {
    const { container } = renderBody("day");
    expect(delays(container)).toEqual(expected(3));
  });
});
