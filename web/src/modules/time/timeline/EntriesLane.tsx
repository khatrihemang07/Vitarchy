import { useMemo } from "react";
import type { LaneEntry } from "@/api/time";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/design";
import { type Clock, formatClock } from "../reports/entryTimes";
import { dayBounds, MIN_WIDTH_PCT, pctOf } from "./bandLayout";

interface EntriesLaneProps {
  day: string;
  entries: readonly LaneEntry[];
  clock: Clock;
}

/** A thin lane (captioned by its parent) above the band with the Day's Time Entries, on the same 24 h scale. */
export function EntriesLane({ day, entries, clock }: EntriesLaneProps) {
  const bounds = useMemo(() => dayBounds(day), [day]);
  return (
    <div className="relative mb-1.5 h-6 rounded-sm bg-muted/50" role="group" aria-label={`Time Entries on this Day: ${String(entries.length)}`}>
      {entries.map((entry) => {
        const left = Math.max(0, pctOf(bounds, Date.parse(entry.start_at)));
        const right = Math.min(100, pctOf(bounds, Date.parse(entry.stop_at)));
        const times = `${formatClock(new Date(entry.start_at), clock)} – ${formatClock(new Date(entry.stop_at), clock)}`;
        const name = entry.description || "(no description)";
        return (
          <Tooltip key={`${entry.start_at}-${entry.stop_at}-${name}`}>
            <TooltipTrigger asChild>
              <button
                type="button"
                aria-label={`${times}, ${name}${entry.project ? `, ${entry.project}` : ""}`}
                className="absolute inset-y-0.5 rounded-sm bg-module/70 outline-none focus-visible:ring-2 focus-visible:ring-ring"
                style={{ left: `${String(left)}%`, width: `${String(Math.max(MIN_WIDTH_PCT, right - left))}%` }}
              />
            </TooltipTrigger>
            <TooltipContent>
              <div className="flex max-w-64 flex-col gap-0.5 break-words">
                <span className="tabular-nums">{times}</span>
                <span className="font-medium">{name}</span>
                <span className="opacity-80">{entry.project ?? "(no project)"}</span>
              </div>
            </TooltipContent>
          </Tooltip>
        );
      })}
    </div>
  );
}
