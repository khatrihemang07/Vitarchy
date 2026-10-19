import type { Overview } from "@/api/finance";
import { overviewTiles } from "./overviewTiles";

const overview: Overview = {
  money_in: { paise: 500_000, previous_paise: 380_000 },
  money_out: { paise: 120_000, previous_paise: 120_000 },
  net: { paise: 380_000, previous_paise: 260_000 },
  transfers_total: { paise: 100_000, previous_paise: 250_000 },
  daily: [],
};

describe("overviewTiles", () => {
  it("lists Money in, Money out, Net and Transfers in that order", () => {
    expect(overviewTiles(overview).map((t) => [t.label, t.value])).toEqual([
      ["Money in", "₹5,000.00"],
      ["Money out", "₹1,200.00"],
      ["Net", "₹3,800.00"],
      ["Transfers", "₹1,000.00"],
    ]);
  });

  it("words each change against the range before, with an arrow", () => {
    const tiles = overviewTiles(overview);
    expect(tiles.map((t) => t.change)).toEqual(["↑ ₹1,200.00", "—", "↑ ₹1,200.00", "↓ ₹1,500.00"]);
    expect(tiles[3]?.delta).toBe(-150_000);
  });

  it("shows a negative Net with a minus sign", () => {
    const tiles = overviewTiles({ ...overview, net: { paise: -5_000, previous_paise: 0 } });
    expect(tiles[2]).toMatchObject({ value: "−₹50.00", change: "↓ ₹50.00" });
  });
});
