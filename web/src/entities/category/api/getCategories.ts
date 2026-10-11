import { api, unwrap, type Schema } from '@/shared/api';

export type BoardIndex = Schema<'BoardIndexResponse'>;
export type CategoryWithBoards = Schema<'CategoryWithBoards'>;
export type BoardSummary = Schema<'BoardSummary'>;

/** `GET /categories`: the board index (admin-only categories and boards are only included for admins). */
export function getCategories(): Promise<BoardIndex> {
    return unwrap(api.GET('/api/v1/categories'));
}
