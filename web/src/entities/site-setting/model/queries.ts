import { useQuery } from '@tanstack/react-query';
import { queryKeys } from '@/shared/api';
import { getSiteSettings } from '../api/site';

export function useSiteSettings() {
    return useQuery({
        queryKey: queryKeys.cms.site(),
        queryFn: getSiteSettings,
    });
}
