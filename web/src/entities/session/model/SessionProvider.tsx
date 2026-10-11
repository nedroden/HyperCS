import { useEffect, useRef, type ReactNode } from 'react';
import { i18n, isLanguageCode } from '@/shared/i18n';
import { DEFAULT_TIME_ZONE, TimeZoneProvider } from '@/shared/lib';
import { useSession } from './useSession';

/**
 * Wires the session into app-wide state, inside `QueryClientProvider`:
 * - the user's time zone feeds `TimeZoneProvider` (UTC for guests);
 * - when a user becomes known (initial load or login) their stored language is applied once. After that
 *   the language switcher owns the value and writes it back to the profile.
 */
export function SessionProvider({ children }: { children: ReactNode }) {
    const { user } = useSession();
    const appliedFor = useRef<string | null>(null);

    useEffect(() => {
        if (!user) {
            appliedFor.current = null;
            return;
        }
        if (appliedFor.current === user.id) return;
        appliedFor.current = user.id;
        if (isLanguageCode(user.language) && i18n.language !== user.language) {
            void i18n.changeLanguage(user.language);
        }
    }, [user]);

    return (
        <TimeZoneProvider timeZone={user?.timezone ?? DEFAULT_TIME_ZONE}>
            {children}
        </TimeZoneProvider>
    );
}
