import { api, unwrap, type Schema } from '@/shared/api';

export type PostDetail = Schema<'PostDetail'>;

/** `GET /posts/{id}`: one post with its topic, board and category context. */
export function getPost(postId: string): Promise<PostDetail> {
    return unwrap(
        api.GET('/api/v1/posts/{id}', { params: { path: { id: postId } } }),
    );
}
