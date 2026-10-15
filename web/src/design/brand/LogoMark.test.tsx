import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { LogoMark } from "./LogoMark";

describe("LogoMark", () => {
  it("exposes an accessible title when given one", () => {
    render(<LogoMark title="Vitarchy" />);
    expect(screen.getByRole("img", { name: "Vitarchy" })).toBeInTheDocument();
  });

  it("is hidden from assistive tech without a title", () => {
    const { container } = render(<LogoMark />);
    expect(container.querySelector("svg")).toHaveAttribute("aria-hidden", "true");
  });

  it("uses unique gradient ids per instance", () => {
    const { container } = render(
      <>
        <LogoMark />
        <LogoMark />
      </>,
    );
    const ids = [...container.querySelectorAll("[id]")].map((el) => el.id);
    expect(ids.length).toBe(6);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("takes every colour from --logo-* tokens, never a hard-coded value", () => {
    const { container } = render(<LogoMark />);
    expect(container.innerHTML).not.toMatch(/#[0-9a-f]{3,8}\b/i);
    expect(container.innerHTML).toContain("var(--logo-pink)");
    expect(container.innerHTML).toContain("var(--logo-lime)");
  });
});
