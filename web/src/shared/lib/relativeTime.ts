import { useTranslation } from 'react-i18next';

type Unit = 'hour' | 'day' | 'week' | 'month';

/** Returns a formatter for relative times in the current language, e.g. `ago(7, 'day')` -> "7 days ago". */
export function useRelativeTime() {
    const { i18n } = useTranslation();
    const rtf = new Intl.RelativeTimeFormat(i18n.language, {
        numeric: 'always',
    });
    return (amount: number, unit: Unit = 'day') => rtf.format(-amount, unit);
}

const UNITS: [Intl.RelativeTimeFormatUnit, number][] = [
    ['year', 365 * 24 * 3600],
    ['month', 30 * 24 * 3600],
    ['day', 24 * 3600],
    ['hour', 3600],
    ['minute', 60],
];

/** Formats `date` relative to `now` with `Intl.RelativeTimeFormat` ("7 days ago", "in 3 hours", "now"). */
export function formatRelative(date: Date, now: Date, locale: string): string {
    const rtf = new Intl.RelativeTimeFormat(locale, { numeric: 'always' });
    const seconds = Math.round((date.getTime() - now.getTime()) / 1000);
    const abs = Math.abs(seconds);
    for (const [unit, size] of UNITS) {
        if (abs >= size) {
            return rtf.format(Math.trunc(seconds / size), unit);
        }
    }
    // Under a minute: "now" (numeric 'auto' gives the localized word).
    return new Intl.RelativeTimeFormat(locale, { numeric: 'auto' }).format(
        0,
        'second',
    );
}

/** Formats an absolute timestamp in the given IANA time zone (falls back to UTC if the zone is invalid). */
export function formatAbsolute(
    date: Date,
    locale: string,
    timeZone: string,
): string {
    const options: Intl.DateTimeFormatOptions = {
        dateStyle: 'long',
        timeStyle: 'short',
    };
    try {
        return new Intl.DateTimeFormat(locale, { ...options, timeZone }).format(
            date,
        );
    } catch {
        return new Intl.DateTimeFormat(locale, {
            ...options,
            timeZone: 'UTC',
        }).format(date);
    }
}
