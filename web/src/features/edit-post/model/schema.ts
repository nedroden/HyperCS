import { z } from 'zod';
import { LIMITS } from '@/shared/config';

export const editPostSchema = z.object({
    body: z
        .string()
        .trim()
        .min(LIMITS.postBody.min, 'errors.required')
        .max(LIMITS.postBody.max, 'errors.too_long'),
});

export type EditPostValues = z.infer<typeof editPostSchema>;
