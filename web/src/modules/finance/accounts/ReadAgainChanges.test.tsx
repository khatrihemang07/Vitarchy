import { screen } from "@testing-library/react";
import type { ReadAgainPreview } from "@/api/finance";
import { preview, renderWithClient, transaction } from "../test-utils";
import { ReadAgainChanges } from "./ReadAgainChanges";

function readAgain(): ReadAgainPreview {
  return {
    statement: preview(),
    sections: [
      {
        position: 0,
        add_count: 0,
        unchanged_count: 1,
        new_printing_count: 2,
        enriched_count: 1,
        no_longer_found_count: 0,
        added: [],
        new_printings: [
          transaction({ description: "Richer row", is_already_stored: true, gains_detail: true }),
          transaction({ description: "Same detail row", is_already_stored: true }),
        ],
      },
    ],
  };
}

describe("ReadAgainChanges", () => {
  it("lists the stored rows that would gain a Printing and tags only those adding detail", () => {
    renderWithClient(<ReadAgainChanges preview={readAgain()} />);

    expect(screen.getByText("Richer row").closest("li")).toHaveTextContent("Gains detail");
    expect(screen.getByText("Same detail row").closest("li")).not.toHaveTextContent("Gains detail");
  });
});
