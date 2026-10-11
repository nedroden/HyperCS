import { useQuery } from '@tanstack/react-query';
import { queryKeys } from '@/shared/api';
import { getTopicPage } from '../api/getTopicPage';

export function useTopicPage(topicId: string, page: number) {
    return useQuery({
        queryKey: queryKeys.topics.detail(topicId, { page }),
        queryFn: () => getTopicPage(topicId, page),
        // Each GET counts as a view, so do not refetch just because the window regained focus.
        refetchOnWindowFocus: false,
    });
}
