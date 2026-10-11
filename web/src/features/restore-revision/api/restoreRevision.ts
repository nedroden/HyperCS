import { api, unwrap } from '@/shared/api';

/** `POST /posts/{id}/revisions/{revisionId}/restore`: writes the old body back as a new revision. */
export function restoreRevision(
    postId: string,
    revisionId: string,
): Promise<void> {
    return unwrap(
        api.POST('/api/v1/posts/{id}/revisions/{revisionId}/restore', {
            params: { path: { id: postId, revisionId } },
        }),
    );
}
