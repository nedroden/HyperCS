import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { formatAbsolute, formatRelative, useTimeZone } from '@/shared/lib';

type RelativeTimeProps = {
    /** ISO 8601 timestamp or Date. */
    date: string | Date;
    /** Reference time; defaults to the current time. Mainly for tests. */
    now?: Date;
};

/**
 * Relative timestamp ("1 week ago") localized with `Intl.RelativeTimeFormat`. The absolute time is in the
 * `title` tooltip, formatted in the user's time zone (see `TimeZoneProvider`, default UTC).
 */
export function RelativeTime({ date, now }: RelativeTimeProps) {
    const { i18n } = useTranslation();
    const timeZone = useTimeZone();
    // Fixed at mount: relative text is not live-updated.
    const [mountedAt] = useState(() => new Date());
    const value = typeof date === 'string' ? new Date(date) : date;
    if (Number.isNaN(value.getTime())) return null;
    return (
        <time
            dateTime={value.toISOString()}
            title={formatAbsolute(value, i18n.language, timeZone)}
        >
            {formatRelative(value, now ?? mountedAt, i18n.language)}
        </time>
    );
}
