import type { Insight } from "@/api/time";
import { insightTarget } from "./insightTarget";

const insight = (subjectKind: Insight["subject"]["kind"], days: string[] = []): Insight => ({
  kind: "outlier",
  subject: { kind: subjectKind, name: "x" },
  measure: "length",
  numbers: { value: 1, samples: 1 },
  score: 1,
  days,
});

describe("insightTarget", () => {
  it("sends screen time to Activity and everything else to the Life log", () => {
    expect(insightTarget(insight("app")).tab).toBe("activity");
    expect(insightTarget(insight("context")).tab).toBe("activity");
    expect(insightTarget(insight("routine_step")).tab).toBe("life-log");
    expect(insightTarget(insight("routine_total")).tab).toBe("life-log");
    expect(insightTarget(insight("description")).tab).toBe("life-log");
  });

  it("opens the Day an Insight names, and otherwise keeps the Period", () => {
    expect(insightTarget(insight("description", ["2026-05-12"])).day).toBe("2026-05-12");
    expect(insightTarget(insight("description")).day).toBeUndefined();
  });
});
