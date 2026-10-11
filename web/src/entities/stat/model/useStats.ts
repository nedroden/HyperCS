import { useQuery } from '@tanstack/react-query';
import { api, queryKeys, unwrap, type Schema } from '@/shared/api';

export type Stats = Schema<'Stats'>;

/** `GET /stats`: totals, newest member, latest post and users online (admin-only content is left out by the API). */
export function useStats() {
    return useQuery({
        queryKey: queryKeys.stats.all,
        queryFn: (): Promise<Stats> => unwrap(api.GET('/api/v1/stats')),
    });
}
