import { useQuery } from '@tanstack/react-query';
import { queryKeys } from '@/shared/api';
import { fetchMe, type Me } from '../api/session';

export type SessionStatus = 'loading' | 'guest' | 'authenticated' | 'error';

export type Session = {
    /** `error`: `/me` failed for another reason than "not logged in" (server down, network). */
    status: SessionStatus;
    /** The logged-in account (incl. role, language, time zone), or `null`. */
    user: Me | null;
    isAdmin: boolean;
    /** True for moderators and admins (everyone who may moderate). */
    isModerator: boolean;
    /** Refetches `/me` (and the CSRF token). */
    refresh: () => Promise<void>;
};

/** Session state from `GET /me` (TanStack Query cache, shared by all callers). Works under any QueryClientProvider. */
export function useSession(): Session {
    const query = useQuery({
        queryKey: queryKeys.me.all,
        queryFn: fetchMe,
        staleTime: 5 * 60_000,
    });
    const user = query.data ?? null;
    const status: SessionStatus = query.isPending
        ? 'loading'
        : user
          ? 'authenticated'
          : query.isError
            ? 'error'
            : 'guest';
    return {
        status,
        user,
        isAdmin: user?.role === 'admin',
        isModerator: user?.role === 'admin' || user?.role === 'moderator',
        refresh: async () => {
            await query.refetch();
        },
    };
}
