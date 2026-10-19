import { render, screen } from "@testing-library/react";
import { Progress } from "./Progress";

describe("Progress", () => {
  it("reports its value when known", () => {
    render(<Progress label="Uploading" value={40} />);
    expect(screen.getByRole("progressbar", { name: "Uploading" })).toHaveAttribute("aria-valuenow", "40");
  });

  it("has no value while the work has no known length", () => {
    render(<Progress label="Reading files" />);
    expect(screen.getByRole("progressbar", { name: "Reading files" })).not.toHaveAttribute("aria-valuenow");
  });

  it("keeps a value within 0 to 100", () => {
    const { container } = render(<Progress label="Over" value={180} />);
    expect(container.querySelector("[style]")).toHaveStyle({ width: "100%" });
  });
});
