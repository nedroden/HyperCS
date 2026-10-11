import type { ProblemDetails } from './http';

/** Thrown by `unwrap` (and so by every query/mutation function) when the API or the network fails. */
export class ApiError extends Error {
    readonly problem: ProblemDetails;

    constructor(problem: ProblemDetails) {
        super(`${problem.code}: ${problem.detail}`);
        this.name = 'ApiError';
        this.problem = problem;
    }

    get status(): number {
        return this.problem.status;
    }

    get code(): string {
        return this.problem.code;
    }
}

/** The problem+json of any thrown value; non-API errors become `unknown_error`. */
export function problemOf(error: unknown): ProblemDetails {
    if (error instanceof ApiError) return error.problem;
    return {
        type: 'about:blank',
        title: 'Error',
        status: 0,
        detail: error instanceof Error ? error.message : 'Unknown error',
        code: 'unknown_error',
    };
}

export function networkProblem(): ProblemDetails {
    return {
        type: 'about:blank',
        title: 'Network error',
        status: 0,
        detail: 'The server could not be reached.',
        code: 'network_error',
    };
}
