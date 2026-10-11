import { z } from 'zod';
import { LANGUAGES } from '@/shared/i18n';
import { LIMITS } from '@/shared/config';

const required = (max: number) =>
    z.string().trim().min(1, 'errors.required').max(max, 'errors.too_long');

export const siteSettingsSchema = z.object({
    siteTitle: required(LIMITS.siteTitle.max),
    slogan: required(LIMITS.slogan.max),
    defaultLanguage: z.enum(
        LANGUAGES.map((l) => l.code) as [string, ...string[]],
        { message: 'errors.invalid_language' },
    ),
});
export type SiteSettingsValues = z.infer<typeof siteSettingsSchema>;
