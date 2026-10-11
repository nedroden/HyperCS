import type { Schema } from '@/shared/api';

export const ALICE_ID = '0190c3a2-0000-7000-8000-000000000001';
export const BOB_ID = '0190c3a2-0000-7000-8000-000000000002';

export const bob: Schema<'UserSummary'> = {
    id: BOB_ID,
    username: 'bob',
    displayName: 'Bob',
    role: 'member',
};

export function makePost(
    overrides: Partial<Schema<'Post'>> = {},
): Schema<'Post'> {
    return {
        id: 'post-1',
        topicId: 'topic-1',
        number: 1,
        body: 'Hello **world**',
        createdAt: '2026-01-01T00:00:00Z',
        author: {
            ...bob,
            joinedAt: '2025-12-01T00:00:00Z',
            postCount: 4,
        },
        ...overrides,
    };
}

export function makeTopic(
    overrides: Partial<Schema<'TopicDetail'>> = {},
): Schema<'TopicDetail'> {
    return {
        id: 'topic-1',
        title: 'Welcome',
        slug: 'welcome',
        boardId: 'board-1',
        boardName: 'General',
        categoryId: 'cat-1',
        categoryName: 'First category',
        createdAt: '2026-01-01T00:00:00Z',
        author: bob,
        firstPostId: 'post-1',
        pinned: false,
        locked: false,
        replyCount: 1,
        viewCount: 3,
        ...overrides,
    };
}

/** Topic page response with a first post by Bob and a reply by Carol. */
export function makeTopicPage(
    topic: Partial<Schema<'TopicDetail'>> = {},
    items?: Schema<'Post'>[],
    paging: Partial<Schema<'PostPage'>> = {},
): Schema<'TopicPageResponse'> {
    const posts = items ?? [
        makePost(),
        makePost({
            id: 'post-2',
            number: 2,
            body: 'A reply',
            author: {
                id: 'carol-id',
                username: 'carol',
                displayName: 'Carol',
                role: 'member',
                joinedAt: '2025-12-01T00:00:00Z',
                postCount: 1,
            },
        }),
    ];
    return {
        topic: makeTopic(topic),
        posts: {
            items: posts,
            page: 1,
            perPage: 15,
            total: posts.length,
            totalPages: 1,
            ...paging,
        },
    };
}

/** `GET /posts/{id}` response: the post plus the context of `makeTopic()`. */
export function makePostDetail(
    post: Schema<'Post'> = makePost(),
    context: Partial<Schema<'PostTopicContext'>> = {},
): Schema<'PostDetail'> {
    return {
        post,
        context: {
            topicId: 'topic-1',
            topicTitle: 'Welcome',
            topicSlug: 'welcome',
            topicLocked: false,
            boardId: 'board-1',
            boardName: 'General',
            categoryId: 'cat-1',
            categoryName: 'First category',
            isFirstPost: post.number === 1,
            page: 1,
            ...context,
        },
    };
}
