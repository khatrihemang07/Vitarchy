import { buildReportQuery } from "./reportQuery";
import { parseReportSearch } from "./reportSearch";

describe("buildReportQuery", () => {
  it("names the Days, time zone and grouping", () => {
    const search = parseReportSearch({ group1: "project", group2: "tag" });
    expect(buildReportQuery(search, { from: "2026-10-12", to: "2026-10-18" }, "Europe/Berlin")).toEqual({
      kind: "week", from: "2026-10-12", to: "2026-10-18", tz: "Europe/Berlin", group1: "project", group2: "tag",
    });
  });

  it("carries filters, and leaves out empty ones", () => {
    const search = parseReportSearch({ project: [1, 2], tag: ["a"], q: "  stand  " });
    const query = buildReportQuery(search, { from: "2026-10-12", to: "2026-10-18" }, "UTC");
    expect(query).toMatchObject({ project: [1, 2], tag: ["a"], q: "stand" });
    expect(query).not.toHaveProperty("client");
  });

  it("sends the Period of the range given, not of the URL", () => {
    const search = parseReportSearch({ kind: "month", date: "2026-02-10" });
    expect(buildReportQuery(search, { from: "2026-02-01", to: "2026-02-28" }, "UTC")).toMatchObject({ kind: "month", from: "2026-02-01", to: "2026-02-28", tz: "UTC" });
  });

  it("leaves out an absent second grouping and empty lists", () => {
    const query = buildReportQuery(parseReportSearch({}), { from: "2026-10-12", to: "2026-10-18" }, "UTC");
    expect(Object.keys(query).sort()).toEqual(["from", "group1", "kind", "to", "tz"]);
  });

  it("omits a blank search", () => {
    expect(buildReportQuery(parseReportSearch({ q: "   " }), { from: "2026-10-12", to: "2026-10-18" }, "UTC")).not.toHaveProperty("q");
  });
});
