import { domainOf, durationSecs, matchesApp, segmentLabel, segmentWhen } from "./segmentRows";
import { summary } from "./testSummary";

const segments = summary({}, "day").segments ?? [];
const at = (i: number) => segments[i] as (typeof segments)[number];
const code = at(0);
const safari = at(1);
const away = at(2);

describe("domainOf", () => {
  it("is the site without www", () => {
    expect(domainOf("https://www.example.com/a/b?q=1")).toBe("example.com");
    expect(domainOf("https://docs.rs/x")).toBe("docs.rs");
  });
  it("is null for no address or one that is not a web page", () => {
    expect(domainOf(null)).toBeNull();
    expect(domainOf("not a url")).toBeNull();
    expect(domainOf("file:///Users/a/b.txt")).toBeNull();
  });
});

describe("durationSecs", () => {
  it("is the Segment's length in seconds", () => {
    expect(durationSecs(code)).toBe(3600);
    expect(durationSecs(safari)).toBe(1800);
  });
});

describe("matchesApp", () => {
  it("accepts everything with no highlight, and only that app with one", () => {
    expect(matchesApp(code, undefined)).toBe(true);
    expect(matchesApp(code, "Code")).toBe(true);
    expect(matchesApp(safari, "Code")).toBe(false);
  });
  it("never matches Away time, which has no app of its own to highlight", () => {
    expect(matchesApp(away, "Code")).toBe(false);
  });
});

describe("segmentLabel", () => {
  it("names the app, or says there is none", () => {
    expect(segmentLabel(code)).toBe("Code");
    expect(segmentLabel({ app: "" })).toBe("(no app)");
  });
});

describe("segmentWhen", () => {
  it("gives the clock times and length, in the chosen clock", () => {
    const when = segmentWhen({ start_at: new Date(2026, 9, 16, 9, 0).toISOString(), end_at: new Date(2026, 9, 16, 9, 30).toISOString() }, "h24");
    expect(when).toBe("09:00 – 09:30 · 30m");
  });
});
