/** The browser's IANA time zone, which Reports and the Timeline are cut in. */
export const browserTimeZone = () => Intl.DateTimeFormat().resolvedOptions().timeZone;
