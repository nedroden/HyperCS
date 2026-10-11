import { keepPreviousData, useQuery } from '@tanstack/react-query';
import { queryKeys, type PageParams } from '@/shared/api';
import { listNews } from '../api/news';

export function useNewsList(params: PageParams = {}) {
    return useQuery({
        queryKey: queryKeys.news.list(params),
        queryFn: () => listNews(params),
        placeholderData: keepPreviousData,
    });
}
