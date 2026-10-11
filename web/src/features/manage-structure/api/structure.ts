import { api, unwrap, type Schema } from '@/shared/api';

export type Category = Schema<'CategoryWithBoards'>;
export type Board = Schema<'BoardSummary'>;
export type Direction = Schema<'Direction'>;
export type CategoryInput = Schema<'CategoryRequest'>;
export type BoardInput = Schema<'BoardRequest'>;

/**
 * `GET /categories`: for admins this includes admin-only categories and boards. Returns the raw
 * response because the query key is shared with the board index (entities/category): one cache
 * entry must always hold one shape. `useStructure` narrows it with `select`.
 */
export function getStructure() {
    return unwrap(api.GET('/api/v1/categories'));
}

export function createCategory(body: CategoryInput) {
    return unwrap(api.POST('/api/v1/categories', { body }));
}

export function updateCategory(id: string, body: CategoryInput) {
    return unwrap(
        api.PATCH('/api/v1/categories/{id}', {
            params: { path: { id } },
            body,
        }),
    );
}

export function deleteCategory(id: string) {
    return unwrap(
        api.DELETE('/api/v1/categories/{id}', { params: { path: { id } } }),
    );
}

export function reorderCategory(id: string, direction: Direction) {
    return unwrap(
        api.POST('/api/v1/categories/{id}/reorder', {
            params: { path: { id } },
            body: { direction },
        }),
    );
}

export function createBoard(body: BoardInput) {
    return unwrap(api.POST('/api/v1/boards', { body }));
}

export function updateBoard(id: string, body: Omit<BoardInput, 'categoryId'>) {
    return unwrap(
        api.PATCH('/api/v1/boards/{id}', { params: { path: { id } }, body }),
    );
}

export function deleteBoard(id: string) {
    return unwrap(
        api.DELETE('/api/v1/boards/{id}', { params: { path: { id } } }),
    );
}

export function reorderBoard(id: string, direction: Direction) {
    return unwrap(
        api.POST('/api/v1/boards/{id}/reorder', {
            params: { path: { id } },
            body: { direction },
        }),
    );
}
