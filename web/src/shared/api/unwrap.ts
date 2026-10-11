import { ApiError, networkProblem } from './error';
import { asProblem } from './http';

type Result<D> = { data?: D; error?: unknown; response: Response };

/**
 * Turns an openapi-fetch result into its data, or throws an `ApiError` carrying the problem+json
 * (network failures become code `network_error`). Use it in every `queryFn`/`mutationFn`:
 * `queryFn: () => unwrap(api.GET('/api/v1/stats'))`. Endpoints without a body (204) resolve to `undefined`.
 */
export async function unwrap<D>(request: Promise<Result<D>>): Promise<D> {
    let result: Result<D>;
    try {
        result = await request;
    } catch {
        throw new ApiError(networkProblem());
    }
    if (result.error !== undefined || !result.response.ok) {
        const problem = asProblem(result.error);
        throw new ApiError({
            ...problem,
            status: problem.status || result.response.status,
        });
    }
    return result.data as D;
}
