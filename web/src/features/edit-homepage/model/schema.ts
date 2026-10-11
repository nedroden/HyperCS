import { z } from 'zod';
import { LIMITS } from '@/shared/config';

const required = (max: number) =>
    z.string().trim().min(1, 'errors.required').max(max, 'errors.too_long');

export const heroSchema = z.object({
    heroHeading: required(LIMITS.heroHeading.max),
    heroButtonLabel: required(LIMITS.heroButton.max),
});
export type HeroValues = z.infer<typeof heroSchema>;

export const featuresSchema = z.object({
    features: z
        .array(
            z.object({
                title: required(LIMITS.featureTitle.max),
                text: required(LIMITS.featureText.max),
            }),
        )
        .length(LIMITS.featureCount),
});
export type FeaturesValues = z.infer<typeof featuresSchema>;

export const termsSchema = z.object({
    body: z
        .string()
        .trim()
        .min(1, 'errors.required')
        .max(LIMITS.termsBody.max, 'errors.too_long'),
});
export type TermsValues = z.infer<typeof termsSchema>;

/** The API reports `features[0].title`; react-hook-form names that field `features.0.title`. */
export function formFieldName(field: string): string {
    return field.replace(/\[(\d+)\]/g, '.$1');
}
