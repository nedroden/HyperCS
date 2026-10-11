import { api, unwrap, type PageParams, type Schema } from '@/shared/api';

export type NewsItem = Schema<'NewsItem'>;
export type NewsItemPage = Schema<'NewsItemPage'>;
export type NewsInput = Schema<'CreateNewsRequest'>;

/** `GET /news`: one page of news items, newest first. */
export function listNews(params: PageParams = {}): Promise<NewsItemPage> {
    return unwrap(api.GET('/api/v1/news', { params: { query: params } }));
}

/** `POST /news` (admin): publishes a news item. */
export function createNews(
    body: NewsInput,
): Promise<Schema<'CreatedResponse'>> {
    return unwrap(api.POST('/api/v1/news', { body }));
}

/** `PATCH /news/{id}` (admin). */
export function updateNews(id: string, body: NewsInput): Promise<void> {
    return unwrap(
        api.PATCH('/api/v1/news/{id}', { params: { path: { id } }, body }),
    );
}

/** `DELETE /news/{id}` (admin). */
export function deleteNews(id: string): Promise<void> {
    return unwrap(
        api.DELETE('/api/v1/news/{id}', { params: { path: { id } } }),
    );
}
