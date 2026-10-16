import type { ReactNode } from "react";
import { toTimeFailure } from "@/api/time";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/design";
import { FailureNotice } from "../FailureNotice";
import { LoadingNotice } from "../LoadingNotice";

/** The part of a query result a panel needs. */
interface PanelQuery<T> {
  isPending: boolean;
  isError: boolean;
  error: unknown;
  data: T | undefined;
}

interface ChartPanelProps<T> {
  title: string;
  description?: string;
  query: PanelQuery<T>;
  /** Whether the loaded data has nothing to draw. */
  isEmpty: (data: T) => boolean;
  emptyMessage: string;
  /** Controls shown in the header, whatever the data. */
  actions?: ReactNode;
  children: (data: T) => ReactNode;
}

/** A card that shows a chart's loading, failure and empty states, and the chart once there is data. */
export function ChartPanel<T>({ title, description, query, isEmpty, emptyMessage, actions, children }: ChartPanelProps<T>) {
  const { data } = query;
  return (
    <Card>
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        {description ? <CardDescription>{description}</CardDescription> : null}
      </CardHeader>
      <CardContent className="flex min-w-0 flex-col gap-4">
        {actions}
        {query.isPending ? <LoadingNotice>Loading…</LoadingNotice> : null}
        {query.isError ? <FailureNotice failure={toTimeFailure(query.error)} /> : null}
        {data !== undefined && isEmpty(data) ? <p className="text-sm text-muted-foreground">{emptyMessage}</p> : null}
        {data !== undefined && !isEmpty(data) ? children(data) : null}
      </CardContent>
    </Card>
  );
}
