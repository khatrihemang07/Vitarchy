import type { ActivityPeriod } from "@/api/time";
import { useRoutineSuggestions } from "@/api/time";
import { ChartPanel } from "../../shared/ChartPanel";
import { useDismissedSuggestions } from "./dismissedSuggestions";
import { suggestionKey } from "./routineModel";
import { SuggestionCard } from "./SuggestionCard";

/** Sequences that keep happening in order, offered as Routines to confirm, rename or dismiss for now. */
export function RoutineSuggestions({ period }: { period: ActivityPeriod }) {
  const query = useRoutineSuggestions(period);
  const { dismissed, dismiss } = useDismissedSuggestions();
  const shown = query.data?.filter((suggestion) => !dismissed.has(suggestionKey(suggestion)));
  return (
    <ChartPanel
      title="Suggested Routines"
      description="Things you logged in the same order on at least five days. Confirm one to follow it; dismissing hides it for this session."
      query={{ ...query, data: shown }}
      isEmpty={(data) => data.length === 0}
      emptyMessage="No new pattern stands out in this Period."
    >
      {(suggestions) => (
        <ul className="grid gap-3 sm:grid-cols-2">
          {suggestions.map((suggestion) => (
            <SuggestionCard key={suggestionKey(suggestion)} suggestion={suggestion} onDismiss={() => { dismiss(suggestionKey(suggestion)); }} />
          ))}
        </ul>
      )}
    </ChartPanel>
  );
}
