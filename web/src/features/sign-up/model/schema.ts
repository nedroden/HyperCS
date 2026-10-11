import { z } from 'zod';
import { LIMITS, USERNAME_PATTERN } from '@/shared/config';

/** Messages are i18n keys (`errors.*`). Limits come from `shared/config` (mirrors the API). */
export const signUpSchema = z
    .object({
        username: z
            .string()
            .trim()
            .min(1, 'errors.required')
            .min(LIMITS.username.min, 'errors.too_short')
            .max(LIMITS.username.max, 'errors.too_long')
            .regex(USERNAME_PATTERN, 'errors.invalid_username'),
        email: z
            .string()
            .trim()
            .min(1, 'errors.required')
            .max(LIMITS.email.max, 'errors.too_long')
            .email('errors.invalid_email'),
        password: z
            .string()
            .min(1, 'errors.required')
            .min(LIMITS.password.min, 'errors.password_too_short')
            .max(LIMITS.password.max, 'errors.password_too_long'),
        confirmPassword: z.string().min(1, 'errors.required'),
        acceptTerms: z.boolean().refine((v) => v, 'errors.terms_not_accepted'),
    })
    .refine((v) => v.password === v.confirmPassword, {
        path: ['confirmPassword'],
        message: 'errors.passwords_mismatch',
    });

export type SignUpValues = z.infer<typeof signUpSchema>;
