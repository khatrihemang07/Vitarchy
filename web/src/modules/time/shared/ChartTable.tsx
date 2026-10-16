interface ChartTableProps {
  caption: string;
  columns: readonly string[];
  rows: readonly (readonly string[])[];
}

/** The chart's numbers as a table, folded away until asked for, so no value depends on colour or hover. */
export function ChartTable({ caption, columns, rows }: ChartTableProps) {
  return (
    <details className="text-sm">
      <summary className="cursor-pointer text-muted-foreground hover:text-foreground">View as table</summary>
      <div className="mt-2 max-h-72 overflow-auto">
        <table className="w-full text-left text-xs tabular-nums">
          <caption className="sr-only">{caption}</caption>
          <thead>
            <tr>
              {columns.map((column) => (
                <th key={column} scope="col" className="sticky top-0 bg-card px-2 py-1 font-medium text-muted-foreground">
                  {column}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {rows.map((row) => (
              <tr key={row.join("|")} className="border-t">
                {row.map((cell, index) => (
                  <td key={columns[index]} className="px-2 py-1">
                    {cell}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </details>
  );
}
