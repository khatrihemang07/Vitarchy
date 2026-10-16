import { render, screen } from "@testing-library/react";
import { Bar, BarChart, XAxis } from "recharts";
import { axisProps } from "./chartTheme";
import { ChartLegend } from "./ChartLegend";
import { ChartTooltip } from "./ChartTooltip";

describe("chart building blocks", () => {
  it("shows a tooltip's title and each formatted row", () => {
    render(<ChartTooltip title="Mon 12 Oct" rows={[{ label: "Office", color: "var(--chart-1)", value: "8 h" }]} />);
    expect(screen.getByRole("tooltip")).toHaveTextContent("Mon 12 Oct");
    expect(screen.getByText("Office")).toBeInTheDocument();
    expect(screen.getByText("8 h")).toBeInTheDocument();
  });

  it("names every series in the legend", () => {
    render(
      <ChartLegend
        items={[
          { label: "Office", color: "var(--chart-1)" },
          { label: "Sleep", color: "var(--chart-2)" },
        ]}
      />,
    );
    expect(screen.getAllByRole("listitem")).toHaveLength(2);
  });

  it("renders two series that share a label without key warnings", () => {
    const error = vi.spyOn(console, "error").mockImplementation(() => undefined);
    render(
      <>
        <ChartLegend items={[{ label: "Code", color: "var(--chart-1)" }, { label: "Code", color: "var(--chart-2)" }]} />
        <ChartTooltip rows={[{ label: "Code", color: "var(--chart-1)", value: "1 h" }, { label: "Code", color: "var(--chart-2)", value: "2 h" }]} />
      </>,
    );
    expect(error).not.toHaveBeenCalled();
    error.mockRestore();
  });

  it("themes a Recharts chart with the shared axis props", () => {
    const { container } = render(
      <BarChart width={240} height={120} data={[{ day: "Mon", hours: 3 }]}>
        <XAxis dataKey="day" {...axisProps} />
        <Bar dataKey="hours" fill="var(--chart-1)" />
      </BarChart>,
    );
    expect(container.querySelector("svg.recharts-surface")).not.toBeNull();
  });
});
