/** Used when the browser cannot list its time zones (`Intl.supportedValuesOf` is missing). */
const FALLBACK_TIME_ZONES = [
    'UTC',
    'Europe/London',
    'Europe/Berlin',
    'Europe/Amsterdam',
    'Europe/Paris',
    'Europe/Madrid',
    'Europe/Rome',
    'Europe/Moscow',
    'Africa/Johannesburg',
    'Asia/Dubai',
    'Asia/Kolkata',
    'Asia/Shanghai',
    'Asia/Tokyo',
    'Australia/Sydney',
    'Pacific/Auckland',
    'America/Sao_Paulo',
    'America/New_York',
    'America/Chicago',
    'America/Denver',
    'America/Los_Angeles',
];

/**
 * All IANA time zones the browser knows, always including `UTC` and `current` (the stored value may be a
 * zone this browser does not list), sorted.
 */
export function timeZoneOptions(current?: string): string[] {
    let zones: string[];
    try {
        zones = Intl.supportedValuesOf('timeZone');
    } catch {
        zones = FALLBACK_TIME_ZONES;
    }
    const all = new Set([...zones, 'UTC']);
    if (current) all.add(current);
    return [...all].sort((a, b) => a.localeCompare(b));
}
