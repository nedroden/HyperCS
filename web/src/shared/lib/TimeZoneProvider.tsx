import type { ReactNode } from 'react';
import { TimeZoneContext } from './timeZoneContext';

/** Provides the user's configured IANA time zone to descendants (see `useTimeZone`). */
export function TimeZoneProvider({
    timeZone,
    children,
}: {
    timeZone: string;
    children: ReactNode;
}) {
    return (
        <TimeZoneContext.Provider value={timeZone}>
            {children}
        </TimeZoneContext.Provider>
    );
}
