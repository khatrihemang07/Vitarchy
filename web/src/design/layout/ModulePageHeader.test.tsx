import { render, screen } from "@testing-library/react";
import { ModulePageHeader } from "./ModulePageHeader";

describe("ModulePageHeader", () => {
  it("shows the title as the page heading, with its subtitle", () => {
    render(<ModulePageHeader title="Reports" subtitle="Where your Time Entries went." />);
    expect(screen.getByRole("heading", { level: 1, name: "Reports" })).toBeInTheDocument();
    expect(screen.getByText("Where your Time Entries went.")).toBeInTheDocument();
  });
});
