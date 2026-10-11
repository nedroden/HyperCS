import { z } from 'zod';
import { LIMITS } from '@/shared/config';

export const preferencesSchema = z.object({
    timezone: z
        .string()
        .min(1, 'errors.required')
        .max(LIMITS.timezone.max, 'errors.too_long'),
    notifyOnReply: z.boolean(),
    notifyOnQuote: z.boolean(),
    showOnline: z.boolean(),
});

export type PreferencesValues = z.infer<typeof preferencesSchema>;
