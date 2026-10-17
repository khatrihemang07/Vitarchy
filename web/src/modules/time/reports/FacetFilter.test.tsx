import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { FacetFilter } from "./FacetFilter";
import type { FacetDescriptor } from "./facetOptions";

const facet = (selected: string[]): FacetDescriptor => ({
  key: "project",
  label: "Project",
  options: [{ value: "1", label: "Office" }, { value: "2", label: "Gym", hint: "archived" }],
  selected,
});

describe("FacetFilter", () => {
  it("shows how many are chosen", () => {
    render(<FacetFilter facet={facet(["1", "2"])} onChange={vi.fn()} />);
    expect(screen.getByRole("button", { name: "Project filter" })).toHaveTextContent("2");
  });

  it("adds a value to the choice", async () => {
    const onChange = vi.fn();
    render(<FacetFilter facet={facet(["1"])} onChange={onChange} />);
    await userEvent.click(screen.getByRole("button", { name: "Project filter" }));
    await userEvent.click(await screen.findByText("Gym"));
    expect(onChange).toHaveBeenCalledWith(["1", "2"]);
  });

  it("removes a value that is already chosen", async () => {
    const onChange = vi.fn();
    render(<FacetFilter facet={facet(["1", "2"])} onChange={onChange} />);
    await userEvent.click(screen.getByRole("button", { name: "Project filter" }));
    await userEvent.click(await screen.findByText("Office"));
    expect(onChange).toHaveBeenCalledWith(["2"]);
  });
});
