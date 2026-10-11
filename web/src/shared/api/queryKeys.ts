/**
 * Query-key conventions. Every key is `[resource, ...scope, params]`, built only here, so a prefix such as
 * `queryKeys.topics.all` invalidates everything about topics. Add new resources to this file (small,
 * append-only edits) and never write literal keys in features or pages.
 */
export type PageParams = { page?: number; perPage?: number };

export const queryKeys = {
    /** The current account (`GET /me`); `null` data means guest. */
    me: {
        all: ['me'] as const,
        sessions: () => ['me', 'sessions'] as const,
    },
    stats: { all: ['stats'] as const },
    /** Board index: `GET /categories`. */
    categories: { all: ['categories'] as const },
    boards: {
        all: ['boards'] as const,
        detail: (id: string, params: PageParams = {}) =>
            ['boards', id, params] as const,
    },
    topics: {
        all: ['topics'] as const,
        detail: (id: string, params: PageParams = {}) =>
            ['topics', id, params] as const,
    },
    posts: {
        all: ['posts'] as const,
        detail: (postId: string) => ['posts', postId, 'detail'] as const,
        revisions: (postId: string, params: PageParams = {}) =>
            ['posts', postId, 'revisions', params] as const,
    },
    users: {
        all: ['users'] as const,
        list: (params: PageParams = {}) => ['users', 'list', params] as const,
        profile: (username: string) => ['users', 'profile', username] as const,
    },
    news: {
        all: ['news'] as const,
        list: (params: PageParams = {}) => ['news', 'list', params] as const,
        detail: (id: string) => ['news', 'detail', id] as const,
    },
    cms: {
        all: ['cms'] as const,
        homepage: () => ['cms', 'homepage'] as const,
        terms: () => ['cms', 'terms'] as const,
        site: () => ['cms', 'site'] as const,
    },
    health: { all: ['health'] as const },
};
