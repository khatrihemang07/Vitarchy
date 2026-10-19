import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { FacetFilter } from "./FacetFilter";

const options = [{ value: "1", label: "Office" }, { value: "2", label: "Gym", hint: "archived" }];
const facet = (selected: string[], onChange = vi.fn()) => <FacetFilter label="Project" options={options} selected={selected} onChange={onChange} />;

describe("FacetFilter", () => {
  it("shows how many are chosen", () => {
    render(facet(["1", "2"]));
    expect(screen.getByRole("button", { name: "Project filter" })).toHaveTextContent("2");
  });

  it("adds a value to the choice", async () => {
    const onChange = vi.fn();
    render(facet(["1"], onChange));
    await userEvent.click(screen.getByRole("button", { name: "Project filter" }));
    await userEvent.click(await screen.findByText("Gym"));
    expect(onChange).toHaveBeenCalledWith(["1", "2"]);
  });

  it("removes a value that is already chosen", async () => {
    const onChange = vi.fn();
    render(facet(["1", "2"], onChange));
    await userEvent.click(screen.getByRole("button", { name: "Project filter" }));
    await userEvent.click(await screen.findByText("Office"));
    expect(onChange).toHaveBeenCalledWith(["2"]);
  });
});
