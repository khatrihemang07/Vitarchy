/** The time-of-day phrase for `date`. */
export function greetingFor(date: Date): string {
  const hour = date.getHours();
  if (hour >= 5 && hour < 12) return "Good morning";
  if (hour >= 12 && hour < 18) return "Good afternoon";
  return "Good evening";
}

/** "Good evening, Hemang", or just "Good evening" without a name. */
export function greeting(date: Date, displayName: string): string {
  const name = displayName.trim();
  return name ? `${greetingFor(date)}, ${name}` : greetingFor(date);
}
