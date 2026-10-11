import { api, unwrap, type Schema } from '@/shared/api';

export type BoardPage = Schema<'BoardPageResponse'>;
export type BoardDetail = Schema<'BoardDetail'>;

/** `GET /boards/{id}`: the board with one page of its topics (pinned first). */
export function getBoardPage(id: string, page: number): Promise<BoardPage> {
    return unwrap(
        api.GET('/api/v1/boards/{id}', {
            params: { path: { id }, query: { page } },
        }),
    );
}
