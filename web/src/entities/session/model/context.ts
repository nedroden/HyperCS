import { createContext } from 'react';

export type SessionUser = { name: string };

export type SessionContextValue = {
    user: SessionUser | null;
    /** Placeholder: signs in a fixed demo user until the API has real authentication. */
    login: () => void;
    logout: () => void;
};

export const SessionContext = createContext<SessionContextValue | null>(null);
