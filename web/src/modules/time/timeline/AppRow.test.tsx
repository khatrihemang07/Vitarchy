import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { AppRow } from "./AppRow";
import { summary } from "./testSummary";

const usage = summary().apps[1] as ReturnType<typeof summary>["apps"][number];

function renderRow(props: Partial<React.ComponentProps<typeof AppRow>> = {}) {
  const onToggleOpen = vi.fn();
  const onHighlight = vi.fn();
  render(
    <ul>
      <AppRow usage={usage} color="red" open={false} highlighted={false} onToggleOpen={onToggleOpen} onHighlight={onHighlight} {...props} />
    </ul>,
  );
  return { onToggleOpen, onHighlight };
}

describe("AppRow", () => {
  it("shows the app with its time and share", () => {
    renderRow();
    expect(screen.getByRole("button", { name: "Safari" })).toBeInTheDocument();
    expect(screen.getByText("45%")).toBeInTheDocument();
  });

  it("asks to expand from the disclosure button and to highlight from the name", async () => {
    const { onToggleOpen, onHighlight } = renderRow();
    await userEvent.click(screen.getByRole("button", { name: "Details of Safari" }));
    expect(onToggleOpen).toHaveBeenCalledOnce();
    expect(onHighlight).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole("button", { name: "Safari" }));
    expect(onHighlight).toHaveBeenCalledOnce();
  });

  it("reflects the open and highlighted states and lists titles and sites when open", () => {
    renderRow({ open: true, highlighted: true });
    expect(screen.getByRole("button", { name: "Details of Safari" })).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByRole("button", { name: "Safari" })).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByText("Window titles")).toBeInTheDocument();
    expect(screen.getByText("docs.rs")).toBeInTheDocument();
  });

  it("says when no titles or sites were recorded", () => {
    renderRow({ open: true, usage: { ...usage, titles: [], sites: [] } });
    expect(screen.getByText("No titles or sites recorded.")).toBeInTheDocument();
  });
});
