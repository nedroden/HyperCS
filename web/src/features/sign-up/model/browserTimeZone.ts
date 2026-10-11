/** The browser's IANA time zone (e.g. "Europe/Amsterdam"), or `undefined` if it cannot be detected. */
export function browserTimeZone(): string | undefined {
    try {
        return Intl.DateTimeFormat().resolvedOptions().timeZone || undefined;
    } catch {
        return undefined;
    }
}
