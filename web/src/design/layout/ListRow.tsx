import type { ReactNode } from "react";

interface ListRowProps {
  /** The main line, such as a file name. */
  title: ReactNode;
  /** A quieter line under it. */
  detail?: ReactNode;
  /** Badges and buttons at the end of the row. */
  children?: ReactNode;
  /** Content under the row, such as a panel the row opens. */
  below?: ReactNode;
}

/** One bordered row of a list (a Statement, a saved password): text on the left, badges and actions on the right. */
export function ListRow({ title, detail, children, below }: ListRowProps) {
  return (
    <li className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center gap-x-3 gap-y-2 rounded-xl border bg-card px-4 py-3">
        <div className="flex min-w-48 flex-1 flex-col gap-0.5">
          <p className="font-medium break-words">{title}</p>
          {detail ? <p className="text-sm text-muted-foreground">{detail}</p> : null}
        </div>
        {children}
      </div>
      {below}
    </li>
  );
}
