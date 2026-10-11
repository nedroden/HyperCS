import { z } from 'zod';
import { LIMITS } from '@/shared/config';

/** Messages are i18n keys (`errors.*`). No composition rules, only length. */
export const changePasswordSchema = z
    .object({
        currentPassword: z.string().min(1, 'errors.required'),
        newPassword: z
            .string()
            .min(1, 'errors.required')
            .min(LIMITS.password.min, 'errors.password_too_short')
            .max(LIMITS.password.max, 'errors.password_too_long'),
        confirmPassword: z.string().min(1, 'errors.required'),
    })
    .refine((v) => v.newPassword === v.confirmPassword, {
        path: ['confirmPassword'],
        message: 'errors.passwords_mismatch',
    });

export type ChangePasswordValues = z.infer<typeof changePasswordSchema>;
