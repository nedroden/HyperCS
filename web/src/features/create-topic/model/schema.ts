import { z } from 'zod';
import { LIMITS } from '@/shared/config';

/** Messages are i18n keys (`errors.*`). Limits mirror the API (`LIMITS`). */
export const createTopicSchema = z.object({
    title: z
        .string()
        .trim()
        .min(1, 'errors.required')
        .min(LIMITS.topicTitle.min, 'errors.too_short')
        .max(LIMITS.topicTitle.max, 'errors.too_long'),
    body: z
        .string()
        .trim()
        .min(LIMITS.postBody.min, 'errors.required')
        .max(LIMITS.postBody.max, 'errors.too_long'),
});

export type CreateTopicValues = z.infer<typeof createTopicSchema>;
