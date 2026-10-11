/**
 * URL builders for the forum routes (see `app/App.tsx`). Topic and board URLs carry an optional cosmetic
 * slug; the id is authoritative, so a missing or wrong slug still resolves.
 */
const slugPart = (slug?: string) =>
    slug ? `/${encodeURIComponent(slug)}` : '';

export const forumIndexPath = '/forum';

export function boardPath(boardId: string, page?: number): string {
    return `/forum/boards/${boardId}${page && page > 1 ? `?page=${page}` : ''}`;
}

export function newTopicPath(boardId: string): string {
    return `/forum/boards/${boardId}/new-topic`;
}

export function topicPath(
    topicId: string,
    slug?: string,
    options: { page?: number; postId?: string } = {},
): string {
    const query =
        options.page && options.page > 1 ? `?page=${options.page}` : '';
    const hash = options.postId ? `#post-${options.postId}` : '';
    return `/forum/topics/${topicId}${slugPart(slug)}${query}${hash}`;
}

export function postEditPath(postId: string): string {
    return `/forum/posts/${postId}/edit`;
}

export function postHistoryPath(postId: string): string {
    return `/forum/posts/${postId}/history`;
}

export function userPath(username: string): string {
    return `/forum/users/${encodeURIComponent(username)}`;
}
