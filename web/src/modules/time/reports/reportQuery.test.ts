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

  it("omits a blank search", () => {
    expect(buildReportQuery(parseReportSearch({ q: "   " }), { from: "2026-10-12", to: "2026-10-18" }, "UTC")).not.toHaveProperty("q");
  });
});
