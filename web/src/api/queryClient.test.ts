import { createQueryClient, FRESH_MS } from "./queryClient";

describe("createQueryClient", () => {
  it("treats a result as fresh for a few seconds, so a warmed page is not fetched twice", () => {
    expect(createQueryClient().getDefaultOptions().queries?.staleTime).toBe(FRESH_MS);
  });
});
