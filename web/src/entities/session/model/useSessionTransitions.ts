import { useQueryClient } from '@tanstack/react-query';
import { queryKeys, setCsrfToken } from '@/shared/api';
import { fetchMe } from '../api/session';

/**
 * Cache updates that must follow login and logout: everything the previous identity could see (admin-only
 * boards, own drafts, ...) is refetched under the new one.
 */
export function useSessionTransitions() {
    const queryClient = useQueryClient();
    const refetchOthers = () =>
        queryClient.invalidateQueries({
            predicate: (q) => q.queryKey[0] !== queryKeys.me.all[0],
        });
    return {
        /** Call after a successful login: loads `/me` (new CSRF token) and refreshes other data. */
        async afterLogin() {
            const me = await fetchMe();
            queryClient.setQueryData(queryKeys.me.all, me);
            await refetchOthers();
        },
        /** Call after logout: the user becomes a guest immediately; other data is refetched as a guest. */
        afterLogout() {
            setCsrfToken(undefined);
            queryClient.setQueryData(queryKeys.me.all, null);
            queryClient.removeQueries({ queryKey: queryKeys.me.sessions() });
            void refetchOthers();
        },
    };
}
