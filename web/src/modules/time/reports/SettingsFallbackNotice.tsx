/** Says the Report is using default Settings because they could not load, and offers to try again. */
export function SettingsFallbackNotice({ onRetry }: { onRetry: () => void }) {
  return (
    <p role="status" className="text-sm text-muted-foreground">
      Using the default week start: Settings could not load.{" "}
      <button type="button" className="underline" onClick={onRetry}>
        Try again
      </button>
    </p>
  );
}
