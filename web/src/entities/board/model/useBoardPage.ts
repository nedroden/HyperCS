import { useQuery } from '@tanstack/react-query';
import { queryKeys } from '@/shared/api';
import { getBoardPage } from '../api/getBoardPage';

export function useBoardPage(boardId: string, page: number) {
    return useQuery({
        queryKey: queryKeys.boards.detail(boardId, { page }),
        queryFn: () => getBoardPage(boardId, page),
    });
}
