import { z } from 'zod';
import { LIMITS } from '@/shared/config';

/** Category and board form. Messages are i18n keys (`errors.*`). */
export const structureSchema = z.object({
    name: z
        .string()
        .trim()
        .min(LIMITS.categoryBoardName.min, 'errors.required')
        .max(LIMITS.categoryBoardName.max, 'errors.too_long'),
    description: z
        .string()
        .trim()
        .max(LIMITS.description.max, 'errors.too_long'),
    adminOnly: z.boolean(),
});

export type StructureValues = z.infer<typeof structureSchema>;
