import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { appColorMap } from "./appColors";
import { AppList } from "./AppList";
import { summary } from "./testSummary";

const { apps } = summary();

function renderList(onApp = vi.fn(), highlight?: string) {
  render(<AppList apps={apps} colors={appColorMap(apps)} highlight={highlight} onApp={onApp} />);
  return onApp;
}

describe("AppList", () => {
  it("ranks apps with their time and share", () => {
    renderList();
    expect(screen.getAllByRole("listitem")[0]).toHaveTextContent("Code");
    expect(screen.getByText("55%")).toBeInTheDocument();
  });

  it("highlights an app when its name is chosen", async () => {
    const onApp = renderList();
    await userEvent.click(screen.getByRole("button", { name: "Safari" }));
    expect(onApp).toHaveBeenCalledWith("Safari");
  });

  it("marks the highlighted app as pressed", () => {
    renderList(vi.fn(), "Code");
    expect(screen.getByRole("button", { name: "Code" })).toHaveAttribute("aria-pressed", "true");
  });

  it("opens a row to its window titles and sites", async () => {
    renderList();
    const toggle = screen.getByRole("button", { name: "Details of Safari" });
    expect(toggle).toHaveAttribute("aria-expanded", "false");
    await userEvent.click(toggle);
    expect(toggle).toHaveAttribute("aria-expanded", "true");
    expect(await screen.findByText("Docs")).toBeInTheDocument();
    expect(screen.getByText("docs.rs")).toBeInTheDocument();
  });
});
