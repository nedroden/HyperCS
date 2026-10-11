import { useQuery } from '@tanstack/react-query';
import { queryKeys } from '@/shared/api';
import { getCategories } from '../api/getCategories';

export function useCategories() {
    return useQuery({
        queryKey: queryKeys.categories.all,
        queryFn: getCategories,
    });
}
