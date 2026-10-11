import { api, unwrap, type Schema } from '@/shared/api';

export type SiteSettings = Schema<'SiteSettings'>;

/** `GET /cms/site`: site title, slogan and default language (public). */
export function getSiteSettings(): Promise<SiteSettings> {
    return unwrap(api.GET('/api/v1/cms/site'));
}

/** `PUT /cms/site` (admin): replaces the site settings. */
export function putSiteSettings(body: SiteSettings): Promise<void> {
    return unwrap(api.PUT('/api/v1/cms/site', { body }));
}
