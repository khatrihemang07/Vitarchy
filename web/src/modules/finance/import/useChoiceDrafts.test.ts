import { act, renderHook } from "@testing-library/react";
import { section } from "../test-utils";
import { useChoiceDrafts } from "./useChoiceDrafts";

const unmatched = section({ matched_account: null });
const matched = section({ matched_account: { id: 9, name: "Existing" } });

describe("useChoiceDrafts", () => {
  it("moves an untouched section to the Account a refreshed preview matched", () => {
    const { result, rerender } = renderHook(({ sections }) => useChoiceDrafts(sections), { initialProps: { sections: [unmatched] } });
    expect(result.current.drafts[0]?.mode).toBe("new");

    rerender({ sections: [matched] });

    expect(result.current.drafts[0]).toEqual({ mode: "existing", accountId: 9 });
    expect(result.current.choices()).toEqual([{ account_id: 9 }]);
  });

  it("keeps what the person entered when the preview is refreshed", () => {
    const { result, rerender } = renderHook(({ sections }) => useChoiceDrafts(sections), { initialProps: { sections: [unmatched] } });
    act(() => {
      result.current.setDraft(0, { mode: "new", account: { issuer: "Own", kind: "savings", name: "Mine" } });
    });

    rerender({ sections: [matched] });

    expect(result.current.drafts[0]).toEqual({ mode: "new", account: { issuer: "Own", kind: "savings", name: "Mine" } });
  });

  it("is ready only when every section says enough", () => {
    const ambiguous = section({ matched_account: null, candidate_accounts: [{ id: 1, name: "A" }, { id: 2, name: "B" }] });
    const { result } = renderHook(() => useChoiceDrafts([matched, ambiguous]));
    expect(result.current.ready).toBe(false);
    act(() => {
      result.current.setDraft(1, { mode: "existing", accountId: 2 });
    });
    expect(result.current.ready).toBe(true);
  });
});
