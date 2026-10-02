import { useMemo, useState, type ReactNode } from 'react';
import {
    SessionContext,
    type SessionContextValue,
    type SessionUser,
} from './context';

export function SessionProvider({ children }: { children: ReactNode }) {
    const [user, setUser] = useState<SessionUser | null>(null);
    const value = useMemo<SessionContextValue>(
        () => ({
            user,
            login: () => setUser({ name: 'Administrator' }),
            logout: () => setUser(null),
        }),
        [user],
    );
    return (
        <SessionContext.Provider value={value}>
            {children}
        </SessionContext.Provider>
    );
}
