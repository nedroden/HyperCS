import { api, unwrap } from '@/shared/api';

/** `DELETE /posts/{id}`. Deleting a topic's first post deletes the whole topic. */
export function deletePost(postId: string): Promise<void> {
    return unwrap(
        api.DELETE('/api/v1/posts/{id}', {
            params: { path: { id: postId } },
        }),
    );
}
