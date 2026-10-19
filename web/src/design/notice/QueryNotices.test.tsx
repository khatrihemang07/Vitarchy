import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryNotices } from "./QueryNotices";

const pending = { isPending: true, isError: false, error: null, refetch: () => Promise.resolve() };
const describeError = () => "Could not load";

describe("QueryNotices", () => {
  beforeEach(() => {
    document.documentElement.style.setProperty("--duration-loading-delay", "250ms");
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
    document.documentElement.style.removeProperty("--duration-loading-delay");
  });

  it("shows nothing before the loading delay and the loading notice after it", () => {
    render(<QueryNotices query={pending} loadingTitle="Loading report…" describeError={describeError} />);
    expect(screen.queryByText("Loading report…")).not.toBeInTheDocument();
    act(() => { vi.advanceTimersByTime(250); });
    expect(screen.getByRole("status")).toHaveTextContent("Loading report…");
  });

  it("shows nothing once the query has data", () => {
    const { container } = render(<QueryNotices query={{ ...pending, isPending: false }} loadingTitle="Loading report…" describeError={describeError} />);
    act(() => { vi.advanceTimersByTime(1000); });
    expect(container).toBeEmptyDOMElement();
  });

  it("shows a failure at once, with Try again", async () => {
    vi.useRealTimers();
    const refetch = vi.fn(() => Promise.resolve());
    render(<QueryNotices query={{ isPending: false, isError: true, error: new Error("x"), refetch }} loadingTitle="Loading report…" describeError={describeError} />);
    expect(screen.getByRole("alert")).toHaveTextContent("Could not load");
    await userEvent.click(screen.getByRole("button", { name: "Try again" }));
    expect(refetch).toHaveBeenCalledTimes(1);
  });
});

describe("QueryNotices entrance step", () => {
  it("renders nothing at all, not even an empty wrapper, during the loading delay", () => {
    vi.useFakeTimers();
    document.documentElement.style.setProperty("--duration-loading-delay", "250ms");
    try {
      const { container } = render(<QueryNotices query={pending} loadingTitle="Loading report…" describeError={describeError} stagger={3} />);
      expect(container).toBeEmptyDOMElement();
      act(() => { vi.advanceTimersByTime(250); });
      expect(container.querySelector(".animate-fade-up")).toContainElement(screen.getByRole("status"));
    } finally {
      vi.useRealTimers();
      document.documentElement.style.removeProperty("--duration-loading-delay");
    }
  });
});
