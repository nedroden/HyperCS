import { api, unwrap, type Schema } from '@/shared/api';

export type HomepageContent = Schema<'HomepageContent'>;
export type TermsContent = Schema<'TermsContent'>;

/** `GET /cms/homepage`: hero and the three feature blocks (the news list comes from `/news`). */
export function getHomepage(): Promise<HomepageContent> {
    return unwrap(api.GET('/api/v1/cms/homepage'));
}

/** `PUT /cms/homepage` (admin): replaces the whole homepage content. */
export function putHomepage(body: HomepageContent): Promise<void> {
    return unwrap(api.PUT('/api/v1/cms/homepage', { body }));
}

/** `GET /cms/terms`: the terms of service text (Markdown / plain text). */
export function getTerms(): Promise<TermsContent> {
    return unwrap(api.GET('/api/v1/cms/terms'));
}

/** `PUT /cms/terms` (admin): replaces the terms of service text. */
export function putTerms(body: TermsContent): Promise<void> {
    return unwrap(api.PUT('/api/v1/cms/terms', { body }));
}
