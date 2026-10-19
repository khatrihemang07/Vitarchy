import { render, screen } from "@testing-library/react";
import { ListRow } from "./ListRow";

describe("ListRow", () => {
  it("shows its title, detail, actions and what sits below", () => {
    render(
      <ul>
        <ListRow title="april.pdf" detail="Uploaded 1 May" below={<p>panel</p>}>
          <button type="button">Review</button>
        </ListRow>
      </ul>,
    );
    expect(screen.getByRole("listitem")).toHaveTextContent("april.pdf");
    expect(screen.getByText("Uploaded 1 May")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Review" })).toBeInTheDocument();
    expect(screen.getByText("panel")).toBeInTheDocument();
  });

  it("leaves out the detail line when there is none", () => {
    render(<ul><ListRow title="only" /></ul>);
    expect(screen.getByRole("listitem").querySelectorAll("p")).toHaveLength(1);
  });
});
