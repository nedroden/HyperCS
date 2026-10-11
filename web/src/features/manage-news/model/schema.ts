import { z } from 'zod';
import { LIMITS } from '@/shared/config';

/** News form; messages are i18n keys (`errors.*`). Limits mirror the API. */
export const newsSchema = z.object({
    title: z
        .string()
        .trim()
        .min(LIMITS.newsTitle.min, 'errors.required')
        .max(LIMITS.newsTitle.max, 'errors.too_long'),
    body: z
        .string()
        .trim()
        .min(1, 'errors.required')
        .max(LIMITS.newsBody.max, 'errors.too_long'),
});

export type NewsValues = z.infer<typeof newsSchema>;
