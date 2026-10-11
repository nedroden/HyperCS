import { z } from 'zod';
import { LIMITS } from '@/shared/config';

/** Messages are i18n keys (`errors.*`). Limits come from `shared/config` (mirrors the API). */
function isHttpUrl(value: string): boolean {
    try {
        const { protocol } = new URL(value);
        return protocol === 'http:' || protocol === 'https:';
    } catch {
        return false;
    }
}

export const profileSchema = z.object({
    displayName: z
        .string()
        .trim()
        .min(LIMITS.displayName.min, 'errors.required')
        .max(LIMITS.displayName.max, 'errors.too_long'),
    location: z.string().trim().max(LIMITS.location.max, 'errors.too_long'),
    website: z
        .string()
        .trim()
        .max(LIMITS.website.max, 'errors.too_long')
        .refine((v) => v === '' || isHttpUrl(v), 'errors.invalid_url'),
    aboutMe: z.string().trim().max(LIMITS.aboutMe.max, 'errors.too_long'),
});
export type ProfileValues = z.infer<typeof profileSchema>;

export const accountSchema = z.object({
    firstName: z.string().trim().max(LIMITS.personName.max, 'errors.too_long'),
    lastName: z.string().trim().max(LIMITS.personName.max, 'errors.too_long'),
    email: z
        .string()
        .trim()
        .min(1, 'errors.required')
        .max(LIMITS.email.max, 'errors.too_long')
        .email('errors.invalid_email'),
});
export type AccountValues = z.infer<typeof accountSchema>;

/** An empty optional field is sent as `null`, which clears it on the server. */
export function nullIfEmpty(value: string): string | null {
    return value === '' ? null : value;
}
