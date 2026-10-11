import { z } from 'zod';
import { LIMITS } from '@/shared/config';

/** Messages are i18n keys (`errors.*`). */
export const signInSchema = z.object({
    username: z
        .string()
        .trim()
        .min(1, 'errors.required')
        .max(LIMITS.username.max, 'errors.too_long'),
    password: z
        .string()
        .min(1, 'errors.required')
        .max(LIMITS.password.max, 'errors.too_long'),
    remember: z.boolean(),
});

export type SignInValues = z.infer<typeof signInSchema>;
