import { Suspense } from "react";
import { LoadingNotice } from "./LoadingNotice";
import { PeriodEmpty } from "./PeriodEmpty";
import { PeriodPicker } from "./PeriodPicker";
import { SourcesTab } from "./SourcesTab";
import { LAZY_BODIES } from "./tabBodies";
import { PERIOD_TABS, TIME_TABS, type TimeTabId } from "./tabs";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/design";
import { useTimeView } from "./useTimeView";

/** `/time`: Overview, Activity and Life log for a Period, and the Sources that feed them. */
export function TimePage() {
  const view = useTimeView();
  const showsPeriod = PERIOD_TABS.includes(view.tab);
  return (
    <div className="flex max-w-5xl flex-col gap-6">
      <header className="animate-fade-up">
        <h1 className="font-display text-4xl font-bold tracking-tight">Time</h1>
        <p className="text-muted-foreground">Your Toggl Time Entries and computer Activity.</p>
      </header>
      <Tabs value={view.tab} onValueChange={(tab) => { view.setTab(tab as TimeTabId); }} className="gap-6">
        <TabsList className="w-full sm:w-fit">
          {TIME_TABS.map((tab) => (
            <TabsTrigger key={tab.id} value={tab.id}>
              {tab.label}
            </TabsTrigger>
          ))}
        </TabsList>
        {showsPeriod ? (
          <PeriodPicker value={view.selection} weekStart={view.weekStart} today={view.today} onChange={view.setSelection} />
        ) : null}
        {TIME_TABS.map((tab) => {
          const Body = LAZY_BODIES[tab.id];
          return (
            <TabsContent key={tab.id} value={tab.id}>
              {Body ? (
                <Suspense fallback={<LoadingNotice>Loading…</LoadingNotice>}>
                  <Body selection={view.selection} />
                </Suspense>
              ) : "empty" in tab ? (
                <PeriodEmpty message={tab.empty} />
              ) : (
                <SourcesTab />
              )}
            </TabsContent>
          );
        })}
      </Tabs>
    </div>
  );
}
