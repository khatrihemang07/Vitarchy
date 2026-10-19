import { useNavigate, useSearch } from "@tanstack/react-router";
import { useCallback, useMemo } from "react";
import { parseTransactionsSearch, type TransactionsSearch, transactionsParams } from "./transactionsSearch";

/** The page state kept in the URL, with a setter that writes it back (only non-default values are kept). */
export function useTransactionsSearch() {
  const raw: Record<string, unknown> = useSearch({ strict: false });
  const search = useMemo(() => parseTransactionsSearch(raw), [raw]);
  const navigate = useNavigate();
  const update = useCallback(
    (next: TransactionsSearch) => {
      void navigate({ to: "/finance/transactions", search: transactionsParams(next) });
    },
    [navigate],
  );
  return { search, update };
}
