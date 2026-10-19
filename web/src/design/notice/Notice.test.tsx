import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { PlugZap } from "lucide-react";
import { Notice } from "./Notice";

describe("Notice", () => {
  it("announces a failure as an alert and anything else as a status", () => {
    const { rerender } = render(<Notice kind="failure" title="Could not reach the server." />);
    expect(screen.getByRole("alert")).toHaveTextContent("Could not reach the server.");
    rerender(<Notice kind="loading" title="Loading report…" />);
    expect(screen.getByRole("status")).toHaveTextContent("Loading report…");
  });

  it("shows its explanation under the title", () => {
    render(<Notice kind="empty" title="No Time Entries">Try another Period.</Notice>);
    expect(screen.getByText("Try another Period.")).toBeInTheDocument();
  });

  it("calls the retry function from Try again", async () => {
    const onRetry = vi.fn();
    render(<Notice kind="failure" title="Failed" onRetry={onRetry} />);
    await userEvent.click(screen.getByRole("button", { name: "Try again" }));
    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it("has no Try again unless asked", () => {
    render(<Notice kind="failure" title="Failed" />);
    expect(screen.queryByRole("button", { name: "Try again" })).not.toBeInTheDocument();
  });

  it("shows another action beside the message", () => {
    render(<Notice kind="warning" title="Careful" action={<a href="/time">Connect</a>} />);
    expect(screen.getByRole("link", { name: "Connect" })).toBeInTheDocument();
  });

  it("spins its icon only for the loading kind", () => {
    const { container, rerender } = render(<Notice kind="loading" title="Loading" />);
    expect(container.querySelector("svg")).toHaveClass("animate-spin");
    rerender(<Notice kind="failure" title="Failed" />);
    expect(container.querySelector("svg")).not.toHaveClass("animate-spin");
  });

  it("renders a kind the Module added, marking it in data-kind", () => {
    const views = { "not-connected": { icon: PlugZap, tone: "bg-module/15 text-module", surface: "", role: "status", spins: false } } as const;
    render(<Notice kind="not-connected" views={views} title="Not connected" />);
    expect(screen.getByRole("status")).toHaveAttribute("data-kind", "not-connected");
  });
});
