import { z } from 'zod';
import { LIMITS } from '@/shared/config';

export const replySchema = z.object({
    body: z
        .string()
        .trim()
        .min(LIMITS.postBody.min, 'errors.required')
        .max(LIMITS.postBody.max, 'errors.too_long'),
});

export type ReplyValues = z.infer<typeof replySchema>;
