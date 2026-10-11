import { api, unwrap, type Schema } from '@/shared/api';

export type TopicPageData = Schema<'TopicPageResponse'>;
export type TopicDetail = Schema<'TopicDetail'>;
export type TopicSummary = Schema<'TopicSummary'>;

/** `GET /topics/{id}`: the topic with one page of posts. Every request counts as a view. */
export function getTopicPage(id: string, page: number): Promise<TopicPageData> {
    return unwrap(
        api.GET('/api/v1/topics/{id}', {
            params: { path: { id }, query: { page } },
        }),
    );
}
