import {
    createNews,
    deleteNews,
    updateNews,
    type NewsInput,
} from '@/entities/news-item';
import { queryKeys, useApiMutation } from '@/shared/api';

const invalidates = [queryKeys.news.all];

export function useNewsMutations() {
    return {
        create: useApiMutation({
            mutationFn: (body: NewsInput) => createNews(body),
            invalidates,
        }),
        update: useApiMutation({
            mutationFn: (v: { id: string; body: NewsInput }) =>
                updateNews(v.id, v.body),
            invalidates,
        }),
        remove: useApiMutation({
            mutationFn: (id: string) => deleteNews(id),
            invalidates,
        }),
    };
}
