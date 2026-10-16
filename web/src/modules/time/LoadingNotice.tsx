/** A quiet placeholder while a card's data loads. */
export function LoadingNotice({ children }: { children: string }) {
  return (
    <p role="status" className="text-sm text-muted-foreground">
      {children}
    </p>
  );
}
