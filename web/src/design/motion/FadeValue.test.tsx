import { render, screen } from "@testing-library/react";
import { FadeValue } from "./FadeValue";
import { MotionProvider } from "./MotionProvider";

const tree = (text: string, reducedMotion: "always" | "never") => (
  <MotionProvider reducedMotion={reducedMotion}>
    <FadeValue text={text} className="big" />
  </MotionProvider>
);

describe("FadeValue", () => {
  it("shows its text at full strength on first mount", () => {
    render(tree("7h 10m", "never"));
    const el = screen.getByText("7h 10m");
    expect(el).toHaveClass("big", "tabular-nums");
    expect(el.style.opacity).not.toBe("0.4");
  });

  it("starts a changed value dimmed, to fade in", () => {
    const view = render(tree("7h 10m", "never"));
    view.rerender(tree("6h 28m", "never"));
    expect(screen.getByText("6h 28m").style.opacity).toBe("0.4");
    expect(screen.queryByText("7h 10m")).not.toBeInTheDocument();
  });

  it("is a plain span, never dimmed, when motion is reduced", () => {
    const view = render(tree("7h 10m", "always"));
    view.rerender(tree("6h 28m", "always"));
    expect(screen.getByText("6h 28m").style.opacity).toBe("");
  });
});
