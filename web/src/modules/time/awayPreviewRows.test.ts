import type { AwayPreview } from "@/api/time";
import { awayPreviewRows, hasPreviewData, watchingPlaces } from "./awayPreviewRows";

const H = 3600;
const preview: AwayPreview = {
  current_secs: 3600,
  watching_cap_secs: 7200,
  watching_apps: ["VLC", "zoom.us"],
  watching_sites: ["youtube.com"],
  days: 28,
  choices: [
    { away_after_secs: 0, away_secs_per_day: 19 * H, active_secs_per_day: 4 * H },
    { away_after_secs: 1800, away_secs_per_day: 17 * H, active_secs_per_day: 6 * H },
    { away_after_secs: 3600, away_secs_per_day: 15 * H, active_secs_per_day: 8 * H },
  ],
};

describe("awayPreviewRows", () => {
  it("names each threshold in minutes, and the hour as an hour", () => {
    expect(awayPreviewRows(preview).map((r) => r.label)).toEqual(["0 min", "30 min", "1 hour"]);
  });

  it("marks only the current threshold", () => {
    expect(awayPreviewRows(preview).map((r) => r.current)).toEqual([false, false, true]);
  });

  it("writes each change against the current threshold, signed", () => {
    const [none, thirty] = awayPreviewRows(preview);
    expect(none).toMatchObject({ away: "19h", awayChange: "+4h", active: "4h", activeChange: "−4h" });
    expect(thirty).toMatchObject({ awayChange: "+2h", activeChange: "−2h" });
  });

  it("calls the current row's change 'current'", () => {
    expect(awayPreviewRows(preview)[2]).toMatchObject({ awayChange: "current", activeChange: "current" });
  });

  it("reads 'same' when another threshold gives the same figure", () => {
    const hour = { away_after_secs: 3600, away_secs_per_day: 15 * H, active_secs_per_day: 8 * H };
    const same = { ...preview, choices: [{ ...hour, away_after_secs: 300 }, hour] };
    expect(awayPreviewRows(same)[0]).toMatchObject({ awayChange: "same", activeChange: "same" });
  });
});

describe("hasPreviewData", () => {
  it("is false when no threshold has any time", () => {
    const empty = { ...preview, choices: preview.choices.map((c) => ({ ...c, away_secs_per_day: 0, active_secs_per_day: 0 })) };
    expect(hasPreviewData(empty)).toBe(false);
    expect(hasPreviewData(preview)).toBe(true);
  });
});

describe("watchingPlaces", () => {
  it("lists the apps, then the sites, as the server sent them", () => {
    expect(watchingPlaces(preview)).toBe("VLC, zoom.us, youtube.com");
  });
});
