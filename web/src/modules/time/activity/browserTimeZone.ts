/** The IANA zone the browser is in: the one Days are measured in on every analysis request. */
export function browserTimeZone(): string {
  return Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC";
}
