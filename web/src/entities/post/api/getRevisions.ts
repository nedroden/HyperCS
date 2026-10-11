import { api, unwrap, type Schema } from '@/shared/api';

export type RevisionPage = Schema<'PostRevisionPage'>;
export type PostRevision = Schema<'PostRevision'>;

/** `GET /posts/{id}/revisions`: the edit history, newest first (author, moderator or admin). */
export function getRevisions(
    postId: string,
    page: number,
): Promise<RevisionPage> {
    return unwrap(
        api.GET('/api/v1/posts/{id}/revisions', {
            params: { path: { id: postId }, query: { page } },
        }),
    );
}
