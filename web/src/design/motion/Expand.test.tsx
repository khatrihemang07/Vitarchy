import { render, screen } from "@testing-library/react";
import { Expand } from "./Expand";
import { MotionProvider } from "./MotionProvider";

function renderExpand(open: boolean, reducedMotion: "always" | "never") {
  const tree = (isOpen: boolean) => (
    <MotionProvider reducedMotion={reducedMotion}>
      <Expand open={isOpen} id="panel-1">
        <p>details</p>
      </Expand>
    </MotionProvider>
  );
  const view = render(tree(open));
  const close = () => {
    view.rerender(tree(false));
  };
  return { close };
}

describe("Expand", () => {
  it("shows its children when open", () => {
    renderExpand(true, "never");
    expect(screen.getByText("details")).toBeInTheDocument();
  });

  it("keeps the element its id names mounted while closed, so aria-controls never dangles", () => {
    renderExpand(false, "never");
    expect(document.getElementById("panel-1")).not.toBeNull();
    expect(screen.queryByText("details")).toBeNull();
  });

  it("removes the children at once, without an exit animation, when motion is reduced", () => {
    const { close } = renderExpand(true, "always");
    close();
    expect(screen.queryByText("details")).toBeNull();
  });

  it("holds the children for the exit animation when motion is full", () => {
    const { close } = renderExpand(true, "never");
    close();
    expect(screen.getByText("details")).toBeInTheDocument();
  });
});
