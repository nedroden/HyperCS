import {
    useMutation,
    useQueryClient,
    type QueryKey,
    type UseMutationOptions,
    type UseMutationResult,
} from '@tanstack/react-query';
import type { ApiError } from './error';

type Options<TData, TVariables> = Omit<
    UseMutationOptions<TData, ApiError, TVariables>,
    'onSuccess'
> & {
    /** Query keys (usually `queryKeys.x.all`) invalidated, and awaited, after the mutation succeeded. */
    invalidates?: readonly QueryKey[];
    onSuccess?: UseMutationOptions<TData, ApiError, TVariables>['onSuccess'];
};

/**
 * `useMutation` with the error typed as `ApiError` (`error.problem` is the problem+json) and declarative
 * invalidation: `mutateAsync` resolves only after the invalidated queries have refetched, so a form can
 * navigate afterwards and see fresh data. Use `problemOf(error)` + `applyProblemToForm` for form errors.
 */
export function useApiMutation<TData = unknown, TVariables = void>({
    invalidates,
    onSuccess,
    ...options
}: Options<TData, TVariables>): UseMutationResult<TData, ApiError, TVariables> {
    const queryClient = useQueryClient();
    return useMutation<TData, ApiError, TVariables>({
        ...options,
        onSuccess: async (...args) => {
            if (invalidates?.length) {
                await Promise.all(
                    invalidates.map((queryKey) =>
                        queryClient.invalidateQueries({ queryKey }),
                    ),
                );
            }
            return onSuccess?.(...args);
        },
    });
}
