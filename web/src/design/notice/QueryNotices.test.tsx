import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryNotices } from "./QueryNotices";

const pending = { isPending: true, isError: false, error: null, refetch: () => Promise.resolve() };
const skeleton = <p>skeleton</p>;
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

  it("shows nothing before the loading delay and the skeleton after it", () => {
    render(<QueryNotices query={pending} skeleton={skeleton} describeError={describeError} />);
    expect(screen.queryByText("skeleton")).not.toBeInTheDocument();
    act(() => { vi.advanceTimersByTime(250); });
    expect(screen.getByText("skeleton")).toBeInTheDocument();
  });

  it("shows nothing once the query has data", () => {
    const { container } = render(<QueryNotices query={{ ...pending, isPending: false }} skeleton={skeleton} describeError={describeError} />);
    act(() => { vi.advanceTimersByTime(1000); });
    expect(container).toBeEmptyDOMElement();
  });

  it("shows a failure at once, with Try again", async () => {
    vi.useRealTimers();
    const refetch = vi.fn(() => Promise.resolve());
    render(<QueryNotices query={{ isPending: false, isError: true, error: new Error("x"), refetch }} skeleton={skeleton} describeError={describeError} />);
    expect(screen.getByRole("alert")).toHaveTextContent("Could not load");
    await userEvent.click(screen.getByRole("button", { name: "Try again" }));
    expect(refetch).toHaveBeenCalledTimes(1);
  });
});
