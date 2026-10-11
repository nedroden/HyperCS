import { createContext, useContext } from 'react';

export const DEFAULT_TIME_ZONE = 'UTC';

export const TimeZoneContext = createContext<string>(DEFAULT_TIME_ZONE);

/** IANA time zone (e.g. "Europe/Amsterdam") used to display absolute timestamps; defaults to UTC. */
export function useTimeZone(): string {
    return useContext(TimeZoneContext);
}
