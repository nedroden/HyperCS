import { keepPreviousData, useQuery } from '@tanstack/react-query';
import { queryKeys, useApiMutation, type PageParams } from '@/shared/api';
import { listUsers, updateRole, type Role } from '../api/users';

export function useUsers(params: PageParams) {
    return useQuery({
        queryKey: queryKeys.users.list(params),
        queryFn: () => listUsers(params),
        placeholderData: keepPreviousData,
    });
}

export function useUpdateRole() {
    return useApiMutation({
        mutationFn: (v: { username: string; role: Role }) =>
            updateRole(v.username, v.role),
        invalidates: [queryKeys.users.all],
    });
}
