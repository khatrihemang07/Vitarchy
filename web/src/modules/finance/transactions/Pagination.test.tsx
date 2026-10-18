import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Pagination } from "./Pagination";

describe("Pagination", () => {
  it("shows the range and pages with Previous and Next", async () => {
    const onPage = vi.fn();
    render(<Pagination total={230} page={2} onPage={onPage} />);
    expect(screen.getByText("51 to 100 of 230")).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Next" }));
    await userEvent.click(screen.getByRole("button", { name: "Previous" }));

    expect(onPage.mock.calls).toEqual([[3], [1]]);
  });

  it("disables Previous on the first page and Next on the last", () => {
    const { rerender } = render(<Pagination total={30} page={1} onPage={() => undefined} />);
    expect(screen.getByRole("button", { name: "Previous" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Next" })).toBeDisabled();
    rerender(<Pagination total={120} page={3} onPage={() => undefined} />);
    expect(screen.getByRole("button", { name: "Next" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Previous" })).toBeEnabled();
  });

  it("says it is loading and holds both buttons while the next page loads", () => {
    render(<Pagination total={230} page={2} onPage={() => undefined} busy />);
    expect(screen.getByRole("status")).toHaveTextContent("Loading Transactions…");
    expect(screen.getByRole("button", { name: "Previous" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Next" })).toBeDisabled();
  });
});
