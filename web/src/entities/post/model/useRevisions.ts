import { useQuery } from '@tanstack/react-query';
import { queryKeys } from '@/shared/api';
import { getRevisions } from '../api/getRevisions';

export function useRevisions(
    postId: string,
    page: number,
    options: { enabled?: boolean } = {},
) {
    return useQuery({
        queryKey: queryKeys.posts.revisions(postId, { page }),
        queryFn: () => getRevisions(postId, page),
        enabled: options.enabled,
    });
}
