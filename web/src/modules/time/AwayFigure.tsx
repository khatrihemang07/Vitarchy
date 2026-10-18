/** One table cell of the Away rules table: a figure and how it differs from the current rule. */
export function AwayFigure({ value, change }: { value: string; change: string }) {
  return (
    <td className="px-2 py-2 text-right">
      <span className="block font-medium">{value}</span>
      <span className="block text-xs text-muted-foreground">{change}</span>
    </td>
  );
}
