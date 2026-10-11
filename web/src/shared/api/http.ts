import createClient from 'openapi-fetch';
import type { components, paths } from './generated/schema';

/** A schema from the OpenAPI document by name: `Schema<'MeResponse'>`. Derive every request/response type from this. */
export type Schema<K extends keyof components['schemas']> =
    components['schemas'][K];

/** Base path of the HyperCS API (proxied to the backend by Vite in development). */
export const API_BASE = '/api/v1';

let csrfToken: string | undefined;

/** Set by the session entity whenever `/me` is fetched or refreshed. */
export function setCsrfToken(token: string | undefined) {
    csrfToken = token;
}

const SAFE_METHODS = new Set(['GET', 'HEAD']);

/** Typed fetch client generated from the OpenAPI schema; never hand-write request/response types. */
export const api = createClient<paths>({
    // Absolute (same origin) because `new Request()` rejects relative URLs outside a browser, e.g. in tests.
    baseUrl: globalThis.location?.origin ?? '',
    credentials: 'same-origin',
    // Resolve `fetch` per request (not at creation) so tests can stub `globalThis.fetch`.
    fetch: (request) => globalThis.fetch(request),
});

api.use({
    onRequest({ request }) {
        if (csrfToken && !SAFE_METHODS.has(request.method.toUpperCase())) {
            request.headers.set('x-csrf-token', csrfToken);
        }
        return request;
    },
});

/** The API's RFC 9457 `application/problem+json` error shape. */
export type ProblemDetails = {
    type: string;
    title: string;
    status: number;
    detail: string;
    code: string;
    errors?: { field: string; code: string; message: string }[];
};

/** Normalizes an openapi-fetch `error` value (of unknown shape) to `ProblemDetails`. */
export function asProblem(error: unknown): ProblemDetails {
    if (error && typeof error === 'object' && 'code' in error) {
        return error as ProblemDetails;
    }
    return {
        type: 'about:blank',
        title: 'Error',
        status: 0,
        detail: 'An unexpected error occurred.',
        code: 'unknown_error',
    };
}
