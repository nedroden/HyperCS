import { useQuery } from '@tanstack/react-query';
import { queryKeys, useApiMutation } from '@/shared/api';
import {
    createBoard,
    createCategory,
    deleteBoard,
    deleteCategory,
    getStructure,
    reorderBoard,
    reorderCategory,
    updateBoard,
    updateCategory,
    type BoardInput,
    type CategoryInput,
    type Direction,
} from '../api/structure';

const invalidates = [queryKeys.categories.all, queryKeys.boards.all];

export function useStructure() {
    return useQuery({
        queryKey: queryKeys.categories.all,
        queryFn: getStructure,
        select: (response) => response.categories,
    });
}

/** All structure mutations; each refetches the board index (and board pages) when it succeeded. */
export function useStructureMutations() {
    return {
        createCategory: useApiMutation({
            mutationFn: (body: CategoryInput) => createCategory(body),
            invalidates,
        }),
        updateCategory: useApiMutation({
            mutationFn: (v: { id: string; body: CategoryInput }) =>
                updateCategory(v.id, v.body),
            invalidates,
        }),
        deleteCategory: useApiMutation({
            mutationFn: (id: string) => deleteCategory(id),
            invalidates,
        }),
        reorderCategory: useApiMutation({
            mutationFn: (v: { id: string; direction: Direction }) =>
                reorderCategory(v.id, v.direction),
            invalidates,
        }),
        createBoard: useApiMutation({
            mutationFn: (body: BoardInput) => createBoard(body),
            invalidates,
        }),
        updateBoard: useApiMutation({
            mutationFn: (v: {
                id: string;
                body: Omit<BoardInput, 'categoryId'>;
            }) => updateBoard(v.id, v.body),
            invalidates,
        }),
        deleteBoard: useApiMutation({
            mutationFn: (id: string) => deleteBoard(id),
            invalidates,
        }),
        reorderBoard: useApiMutation({
            mutationFn: (v: { id: string; direction: Direction }) =>
                reorderBoard(v.id, v.direction),
            invalidates,
        }),
    };
}
