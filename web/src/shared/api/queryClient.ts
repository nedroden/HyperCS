import {
    MutationCache,
    QueryCache,
    QueryClient,
    type QueryKey,
} from '@tanstack/react-query';
import { ApiError } from './error';
import { queryKeys } from './queryKeys';

/** Codes of 401 responses that do not mean "your session ended". */
const NOT_A_SESSION_LOSS = new Set(['invalid_credentials']);

/**
 * Creates the app's QueryClient. 4xx errors are not retried; a 401 on an authenticated request refetches
 * `/me` so the UI notices an expired session. Tests pass `{ retry: false }`-like defaults via this too.
 */
export function createQueryClient(): QueryClient {
    const onError = (error: unknown) => {
        if (
            error instanceof ApiError &&
            error.status === 401 &&
            !NOT_A_SESSION_LOSS.has(error.code)
        ) {
            void client.invalidateQueries({ queryKey: queryKeys.me.all });
        }
    };
    const client: QueryClient = new QueryClient({
        queryCache: new QueryCache({ onError }),
        mutationCache: new MutationCache({ onError }),
        defaultOptions: {
            queries: {
                retry: (count, error) =>
                    error instanceof ApiError &&
                    error.status >= 400 &&
                    error.status < 500
                        ? false
                        : count < 2,
            },
        },
    });
    return client;
}

export type { QueryKey };
