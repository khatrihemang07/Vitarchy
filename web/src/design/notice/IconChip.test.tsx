import { render } from "@testing-library/react";
import { Inbox } from "lucide-react";
import { IconChip } from "./IconChip";

describe("IconChip", () => {
  it("spins its icon only when asked", () => {
    const { container, rerender } = render(<IconChip icon={Inbox} tone="bg-muted" spins />);
    expect(container.querySelector("svg")).toHaveClass("animate-spin");
    rerender(<IconChip icon={Inbox} tone="bg-muted" />);
    expect(container.querySelector("svg")).not.toHaveClass("animate-spin");
  });

  it("is smaller when small", () => {
    const { container } = render(<IconChip icon={Inbox} tone="bg-muted" small />);
    expect(container.firstChild).toHaveClass("size-9");
  });
});
