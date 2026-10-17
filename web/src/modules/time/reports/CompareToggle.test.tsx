import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { CompareToggle } from "./CompareToggle";

describe("CompareToggle", () => {
  it("offers the Previous Period and the last 4 weeks, marks the current one and reports a change", async () => {
    const onChange = vi.fn();
    render(<CompareToggle compare="usual" onChange={onChange} />);
    expect(screen.getByRole("radio", { name: "Last 4 weeks" })).toBeChecked();
    await userEvent.click(screen.getByRole("radio", { name: "Previous Period" }));
    expect(onChange).toHaveBeenCalledWith("previous");
  });
});
