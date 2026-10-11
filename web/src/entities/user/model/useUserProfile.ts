import { useQuery } from '@tanstack/react-query';
import { queryKeys } from '@/shared/api';
import { getUserProfile } from '../api/getUserProfile';

export function useUserProfile(username: string) {
    return useQuery({
        queryKey: queryKeys.users.profile(username),
        queryFn: () => getUserProfile(username),
    });
}
