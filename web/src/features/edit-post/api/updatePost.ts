import { api, unwrap } from '@/shared/api';

/** `PATCH /posts/{id}`: edits a post's Markdown body (the old body is kept as a revision). */
export function updatePost(postId: string, body: string): Promise<void> {
    return unwrap(
        api.PATCH('/api/v1/posts/{id}', {
            params: { path: { id: postId } },
            body: { body },
        }),
    );
}
