/**
 * Validation limits, mirrored from the API (`api/crates/domain/src/constants.rs`, which is also what the
 * OpenAPI schema's min/max are generated from). Use these in zod schemas, `maxLength` attributes and hints;
 * never write the numbers inline. Keep in sync with the API when it changes.
 */
export const LIMITS = {
    username: { min: 3, max: 32 },
    displayName: { min: 1, max: 50 },
    email: { max: 254 },
    password: { min: 10, max: 128 },
    personName: { max: 60 },
    location: { max: 100 },
    timezone: { max: 64 },
    topicTitle: { min: 3, max: 120 },
    postBody: { min: 1, max: 10_000 },
    categoryBoardName: { min: 1, max: 60 },
    description: { max: 200 },
    aboutMe: { max: 500 },
    website: { max: 200 },
    newsTitle: { min: 1, max: 120 },
    newsBody: { max: 5_000 },
    heroHeading: { max: 120 },
    siteTitle: { max: 60 },
    slogan: { max: 300 },
    heroButton: { max: 60 },
    featureTitle: { max: 60 },
    featureText: { max: 200 },
    featureCount: 3,
    termsBody: { max: 20_000 },
} as const;

/** Allowed username characters (the API also reserves some names and reports them as `username_taken`). */
export const USERNAME_PATTERN = /^[A-Za-z0-9_-]+$/;

/** Page sizes the API uses by default (informational; the API enforces them). */
export const PAGE_SIZES = {
    topics: 20,
    posts: 15,
    news: 10,
    users: 20,
} as const;
