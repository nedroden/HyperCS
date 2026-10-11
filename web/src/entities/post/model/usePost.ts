import { useQuery } from '@tanstack/react-query';
import { queryKeys } from '@/shared/api';
import { getPost } from '../api/getPost';

/** Loads a post (with its topic context) by id, so the page works on a direct visit or reload. */
export function usePost(postId: string | undefined) {
    return useQuery({
        queryKey: queryKeys.posts.detail(postId ?? ''),
        queryFn: () => getPost(postId ?? ''),
        enabled: !!postId,
    });
}
