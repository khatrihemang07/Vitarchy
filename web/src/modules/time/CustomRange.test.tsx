import { fireEvent, render, screen } from "@testing-library/react";
import { CustomRange } from "./CustomRange";
import type { PeriodSelection } from "./period";

const value: PeriodSelection = { kind: "custom", from: "2026-10-05", to: "2026-10-09" };

function setup(current = value) {
  const onChange = vi.fn();
  render(<CustomRange value={current} max="2026-10-14" onChange={onChange} />);
  return { onChange, from: screen.getByLabelText("From"), to: screen.getByLabelText("To") };
}

describe("CustomRange", () => {
  it("applies a valid edit", () => {
    const { onChange, to } = setup();
    fireEvent.change(to, { target: { value: "2026-10-12" } });
    expect(onChange).toHaveBeenLastCalledWith({ kind: "custom", from: "2026-10-05", to: "2026-10-12" });
  });

  it("moves the end along when the start is set after it", () => {
    const { onChange, from, to } = setup();
    fireEvent.change(from, { target: { value: "2026-10-11" } });
    expect(onChange).toHaveBeenLastCalledWith({ kind: "custom", from: "2026-10-11", to: "2026-10-11" });
    expect(to).toHaveValue("2026-10-11");
  });

  it("moves the start back when the end is set before it", () => {
    const { onChange, from } = setup();
    fireEvent.change(screen.getByLabelText("To"), { target: { value: "2026-10-02" } });
    expect(onChange).toHaveBeenLastCalledWith({ kind: "custom", from: "2026-10-02", to: "2026-10-02" });
    expect(from).toHaveValue("2026-10-02");
  });

  it("keeps a cleared field empty while the person retypes, without applying it", () => {
    const { onChange, from } = setup();
    fireEvent.change(from, { target: { value: "" } });
    expect(from).toHaveValue("");
    expect(onChange).not.toHaveBeenCalled();
    fireEvent.change(from, { target: { value: "2026-10-03" } });
    expect(onChange).toHaveBeenLastCalledWith({ kind: "custom", from: "2026-10-03", to: "2026-10-09" });
  });

  it("explains and does not apply a range longer than 400 days", () => {
    const { onChange, from } = setup();
    fireEvent.change(from, { target: { value: "2024-01-01" } });
    expect(screen.getByRole("alert")).toHaveTextContent("400 days");
    expect(onChange).not.toHaveBeenCalled();
  });

  it("explains and does not apply a date after today", () => {
    const { onChange, to } = setup();
    fireEvent.change(to, { target: { value: "2026-10-20" } });
    expect(screen.getByRole("alert")).toHaveTextContent(/future|today/i);
    expect(onChange).not.toHaveBeenCalled();
  });
});
