import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render } from "@testing-library/react";
import type { ReactElement } from "react";
import type { AccountSummary, Overview, SectionPreview, StatementPreview, TransactionView } from "@/api/finance";
import { TooltipProvider } from "@/design";

/** A `Response` carrying JSON (problem documents use the problem content type). */
export function jsonResponse(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": status >= 400 ? "application/problem+json" : "application/json" },
  });
}

type Handler = (request: Request) => Response | Promise<Response>;

/**
 * Stubs `fetch` with handlers keyed `"METHOD /path"`. A handler may be a list:
 * each call takes the next entry, the last one repeating.
 */
export function stubApi(routes: Record<string, Handler | Handler[]>) {
  const counters = new Map<string, number>();
  const fetchMock = vi.fn((request: Request) => {
    const key = `${request.method} ${new URL(request.url).pathname}`;
    const entry = routes[key];
    if (!entry) return Promise.resolve(jsonResponse(404, { detail: `no stub for ${key}` }));
    const handlers = Array.isArray(entry) ? entry : [entry];
    const index = counters.get(key) ?? 0;
    counters.set(key, index + 1);
    const handler = handlers.at(Math.min(index, handlers.length - 1));
    return Promise.resolve(handler ? handler(request) : jsonResponse(404, { detail: "no handler" }));
  });
  vi.stubGlobal("fetch", fetchMock);
  return fetchMock;
}

/** The `"METHOD /path"` of every request a stub saw, in order. */
export function requestsOf(fetchMock: ReturnType<typeof stubApi>): string[] {
  return fetchMock.mock.calls.map(([request]) => `${request.method} ${new URL(request.url).pathname}`);
}

/** Renders `ui` with a fresh query cache and the tooltip provider the app mounts at its root. */
export function renderWithClient(ui: ReactElement) {
  return render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <TooltipProvider>{ui}</TooltipProvider>
    </QueryClientProvider>,
  );
}

/** A debit Transaction as a preview shows it. */
export function transaction(over: Partial<TransactionView> = {}): TransactionView {
  return {
    date: "2026-04-01",
    description: "UPI payment to a shop",
    amount_paise: 123_456,
    direction: "debit",
    balance_paise: 9_876_500,
    is_already_stored: false,
    gains_detail: false,
    ...over,
  };
}

/** One Account section of a preview with two new rows and one stored one. */
export function section(over: Partial<SectionPreview> = {}): SectionPreview {
  return {
    position: 0,
    proposed: { issuer: "HDFC", kind: "savings", mask: "XXXX1234" },
    matched_account: { id: 1, name: "HDFC Savings" },
    matched_without_number: false,
    candidate_accounts: [],
    period: { start: "2026-04-01", end: "2026-04-30" },
    transaction_count: 3,
    total_debit_paise: 200_000,
    total_credit_paise: 50_000,
    checks: [
      { name: "balance_chain", outcome: "passed", detail: {} },
      { name: "summary_totals", outcome: "failed", detail: {} },
      { name: "row_counts", outcome: "skipped", detail: {} },
    ],
    new_count: 2,
    already_stored_count: 1,
    enriched_count: 0,
    transactions: [
      transaction({ description: "First row" }),
      transaction({ description: "Second row", direction: "credit" }),
      transaction({ description: "Stored row", is_already_stored: true }),
    ],
    ...over,
  };
}

/** A read, unconfirmed PDF Statement with one section. */
export function preview(over: Partial<StatementPreview> = {}): StatementPreview {
  return {
    id: 1,
    file_name: "april.pdf",
    format: "pdf",
    kind: "statement",
    outcome: "read",
    status: "unconfirmed",
    duplicate_file: false,
    diagnostics: { stage: null, code: null, pages: 2, rows: 3 },
    period: { start: "2026-04-01", end: "2026-04-30" },
    encrypted: false,
    unlocked_with_label: null,
    uploaded_at: "2026-05-01T10:00:00Z",
    sections: [section()],
    ...over,
  };
}

/** An Account with its figures. */
export function account(over: Partial<AccountSummary> = {}): AccountSummary {
  return {
    id: 1,
    issuer: "HDFC",
    kind: "savings",
    mask: "XXXX1234",
    name: "HDFC Savings",
    hidden: false,
    latest_balance_paise: 9_876_500,
    transaction_count: 12,
    first_date: "2026-01-01",
    last_date: "2026-04-30",
    statement_count: 2,
    ...over,
  };
}

/** A handler whose request never settles: the query stays pending. */
export const pendingForever = () => new Promise<Response>(() => undefined);

/** Every notice kind that must stay absent while queries are still loading. */
export const SETTLED_NOTICES = '[data-kind="warning"], [data-kind="empty"], [data-kind="failure"]';

/** An Overview of two days with a rise in Money in and no change in Money out. */
export function overview(over: Partial<Overview> = {}): Overview {
  return {
    money_in: { paise: 500_000, previous_paise: 380_000 },
    money_out: { paise: 120_000, previous_paise: 120_000 },
    net: { paise: 380_000, previous_paise: 260_000 },
    transfers_total: { paise: 100_000, previous_paise: 250_000 },
    daily: [
      { date: "2026-04-01", money_in_paise: 500_000, money_out_paise: 0 },
      { date: "2026-04-02", money_in_paise: 0, money_out_paise: 120_000 },
    ],
    ...over,
  };
}
