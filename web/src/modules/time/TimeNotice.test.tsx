import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { TimeNotice } from "./TimeNotice";

describe("TimeNotice", () => {
  it("announces a failure as an alert and anything else as a status", () => {
    const { rerender } = render(<TimeNotice kind="failure" title="Could not reach the server." />);
    expect(screen.getByRole("alert")).toHaveTextContent("Could not reach the server.");
    rerender(<TimeNotice kind="loading" title="Loading report…" />);
    expect(screen.getByRole("status")).toHaveTextContent("Loading report…");
  });

  it("shows its explanation under the title", () => {
    render(<TimeNotice kind="empty" title="No Time Entries">Try another Period.</TimeNotice>);
    expect(screen.getByText("Try another Period.")).toBeInTheDocument();
  });

  it("calls the retry function from Try again", async () => {
    const onRetry = vi.fn();
    render(<TimeNotice kind="failure" title="Failed" onRetry={onRetry} />);
    await userEvent.click(screen.getByRole("button", { name: "Try again" }));
    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it("has no Try again unless asked", () => {
    render(<TimeNotice kind="failure" title="Failed" />);
    expect(screen.queryByRole("button", { name: "Try again" })).not.toBeInTheDocument();
  });

  it("shows another action beside the message", () => {
    render(<TimeNotice kind="not-connected" title="Not connected" action={<a href="/time">Connect</a>} />);
    expect(screen.getByRole("link", { name: "Connect" })).toBeInTheDocument();
  });
});

describe("TimeNotice icon", () => {
  it("spins only for the loading kind", () => {
    const { container, rerender } = render(<TimeNotice kind="loading" title="Loading" />);
    expect(container.querySelector("svg")).toHaveClass("animate-spin");
    rerender(<TimeNotice kind="failure" title="Failed" />);
    expect(container.querySelector("svg")).not.toHaveClass("animate-spin");
  });
});
