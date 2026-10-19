import { act, fireEvent, render, screen } from "@testing-library/react";
import { DescriptionSearch } from "./DescriptionSearch";

describe("DescriptionSearch", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("reports the text once typing pauses, not on every keystroke", () => {
    const onSearch = vi.fn();
    render(<DescriptionSearch value="" onSearch={onSearch} />);
    const box = screen.getByRole("searchbox", { name: "Search descriptions" });

    fireEvent.change(box, { target: { value: "r" } });
    fireEvent.change(box, { target: { value: "rent" } });
    expect(onSearch).not.toHaveBeenCalled();
    act(() => { vi.advanceTimersByTime(300); });

    expect(onSearch).toHaveBeenCalledOnce();
    expect(onSearch).toHaveBeenCalledWith("rent");
  });

  it("shows a value that changed elsewhere, such as Clear filters", () => {
    const { rerender } = render(<DescriptionSearch value="rent" onSearch={vi.fn()} />);
    expect(screen.getByRole("searchbox")).toHaveValue("rent");
    rerender(<DescriptionSearch value="" onSearch={vi.fn()} />);
    expect(screen.getByRole("searchbox")).toHaveValue("");
  });

  it("does not report after it is gone", () => {
    const onSearch = vi.fn();
    const { unmount } = render(<DescriptionSearch value="" onSearch={onSearch} />);
    fireEvent.change(screen.getByRole("searchbox"), { target: { value: "x" } });
    unmount();
    act(() => { vi.advanceTimersByTime(500); });
    expect(onSearch).not.toHaveBeenCalled();
  });
});
