import { render, screen } from "@testing-library/react";
import { SectionCard } from "./SectionCard";

describe("SectionCard", () => {
  it("names its region after the title", () => {
    render(<SectionCard title="Summary"><p>rows</p></SectionCard>);
    expect(screen.getByRole("region", { name: "Summary" })).toHaveTextContent("rows");
  });
});
