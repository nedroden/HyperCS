import { useQuery } from '@tanstack/react-query';
import { getHealth } from '../api/getHealth';

export type ApiStatus = 'loading' | 'ok' | 'down';

export function useApiHealth(): ApiStatus {
    const { isPending, isError } = useQuery({
        queryKey: ['api-health'],
        queryFn: getHealth,
        retry: false,
    });
    return isPending ? 'loading' : isError ? 'down' : 'ok';
}
