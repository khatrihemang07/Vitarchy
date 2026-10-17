import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { StatModeToggle } from "./StatModeToggle";

describe("StatModeToggle", () => {
  it("marks the current mode and reports a change", async () => {
    const onChange = vi.fn();
    render(<StatModeToggle mode="per_day" onChange={onChange} />);
    expect(screen.getByRole("radio", { name: "Per Day" })).toBeChecked();
    await userEvent.click(screen.getByRole("radio", { name: "Per entry" }));
    expect(onChange).toHaveBeenCalledWith("per_entry");
  });
});
