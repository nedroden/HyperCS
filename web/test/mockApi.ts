import { vi } from 'vitest';
import type { paths } from '@/shared/api/generated/schema';
import type { ProblemDetails } from '@/shared/api';

/**
 * A typed fake of the API at the `fetch` boundary (msw is not installed; `vi.stubGlobal('fetch')` is enough
 * because the app only talks to `/api/v1` through one openapi-fetch client). Paths, methods, status codes
 * and response bodies are all checked against the generated OpenAPI types:
 *
 *   const mock = mockApi();
 *   mock.on('get', '/api/v1/stats', 200, { topics: 1, ... });
 *   mock.on('post', '/api/v1/auth/login', 204);
 *   mock.on('post', '/api/v1/auth/login', 401, problem(401, 'invalid_credentials'));
 *   expect(mock.calls('post', '/api/v1/auth/login')[0].body).toEqual({ ... });
 *
 * Later registrations win over earlier ones for the same route. Requests nobody handled get a 501 problem
 * and fail the test in `afterEach` (see `test/setup.ts`), so a forgotten endpoint is never silent.
 */

type Method = 'get' | 'post' | 'put' | 'patch' | 'delete';
type Path = keyof paths;
type MethodsOf<P extends Path> = {
    [M in Method]: paths[P][M] extends { responses: unknown } ? M : never;
}[Method];
type ResponsesOf<P extends Path, M extends MethodsOf<P>> = paths[P][M] extends {
    responses: infer R;
}
    ? R
    : never;
type StatusOf<P extends Path, M extends MethodsOf<P>> = keyof ResponsesOf<
    P,
    M
> &
    number;
type ContentOf<R> = R extends { content: infer C } ? C[keyof C] : undefined;
type BodyOf<
    P extends Path,
    M extends MethodsOf<P>,
    S extends StatusOf<P, M>,
> = ContentOf<ResponsesOf<P, M>[S]>;

export type RecordedCall = {
    method: Uppercase<Method>;
    /** The URL path, e.g. `/api/v1/topics/123`. */
    path: string;
    /** `{id}` placeholders of the matched route. */
    params: Record<string, string>;
    search: URLSearchParams;
    headers: Headers;
    /** Parsed JSON request body, if any. */
    body: unknown;
};

type Route = {
    method: Uppercase<Method>;
    template: string;
    regex: RegExp;
    names: string[];
    status: number;
    body: unknown;
};

function compile(template: string): { regex: RegExp; names: string[] } {
    const names: string[] = [];
    const source = template
        .split(/(\{[^}]+\})/)
        .map((part) => {
            if (part.startsWith('{')) {
                names.push(part.slice(1, -1));
                return '([^/]+)';
            }
            return part.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
        })
        .join('');
    return { regex: new RegExp(`^${source}/?$`), names };
}

export type MockApi = ReturnType<typeof mockApi>;

let active: MockApi | undefined;

export function mockApi() {
    const routes: Route[] = [];
    const recorded: RecordedCall[] = [];
    const unhandled: string[] = [];

    async function handle(input: Request | string | URL, init?: RequestInit) {
        const request = new Request(input, init);
        const url = new URL(request.url, 'http://localhost');
        const method = request.method.toUpperCase() as Uppercase<Method>;
        let body: unknown;
        if (method !== 'GET' && method !== 'DELETE') {
            const text = await request.clone().text();
            try {
                body = text ? JSON.parse(text) : undefined;
            } catch {
                body = text;
            }
        }
        const route = [...routes]
            .reverse()
            .find((r) => r.method === method && r.regex.test(url.pathname));
        if (!route) {
            unhandled.push(`${method} ${url.pathname}`);
            return json(501, problem(501, 'unhandled_test_request'));
        }
        const match = route.regex.exec(url.pathname) ?? [];
        const params = Object.fromEntries(
            route.names.map((name, i) => [
                name,
                decodeURIComponent(match[i + 1]),
            ]),
        );
        recorded.push({
            method,
            path: url.pathname,
            params,
            search: url.searchParams,
            headers: request.headers,
            body,
        });
        return json(route.status, route.body);
    }

    const controller = {
        /** Registers a response for a route (path template as in the OpenAPI document, e.g. `/api/v1/boards/{id}`). */
        on<P extends Path, M extends MethodsOf<P>, S extends StatusOf<P, M>>(
            method: M,
            path: P,
            status: S,
            ...body: BodyOf<P, M, S> extends undefined ? [] : [BodyOf<P, M, S>]
        ): void {
            const { regex, names } = compile(path);
            routes.push({
                method: method.toUpperCase() as Uppercase<Method>,
                template: path,
                regex,
                names,
                status,
                body: body[0],
            });
        },
        /**
         * Like `on`, for responses the OpenAPI document does not list (e.g. a 500 or 503 on any path), to test
         * failure handling. The path is still checked; status and body are not.
         */
        onUnlisted<P extends Path>(
            method: MethodsOf<P>,
            path: P,
            status: number,
            body?: unknown,
        ): void {
            const { regex, names } = compile(path);
            routes.push({
                method: method.toUpperCase() as Uppercase<Method>,
                template: path,
                regex,
                names,
                status,
                body,
            });
        },
        /** Requests the route received, oldest first. */
        calls<P extends Path, M extends MethodsOf<P>>(
            method: M,
            path: P,
        ): RecordedCall[] {
            const { regex } = compile(path);
            return recorded.filter(
                (c) => c.method === method.toUpperCase() && regex.test(c.path),
            );
        },
        /** All recorded requests, oldest first. */
        get allCalls(): readonly RecordedCall[] {
            return recorded;
        },
        /** Requests that matched no registered route. */
        get unhandled(): readonly string[] {
            return unhandled;
        },
    };

    // Every screen reads the site settings (header, document title); tests can register their own.
    controller.on('get', '/api/v1/cms/site', 200, {
        siteTitle: 'HyperCS',
        slogan: 'Slogan',
        defaultLanguage: 'en-US',
    });

    vi.stubGlobal('fetch', handle);
    active = controller;
    return controller;
}

function json(status: number, body: unknown): Response {
    if (status === 204 || body === undefined) {
        return new Response(null, { status });
    }
    return new Response(JSON.stringify(body), {
        status,
        headers: {
            'content-type':
                status >= 400 ? 'application/problem+json' : 'application/json',
        },
    });
}

/** Builds an RFC 9457 problem body: `problem(422, 'validation_failed', [{ field: 'email', code: 'email' }])`. */
export function problem(
    status: number,
    code: string,
    errors?: { field: string; code: string; message?: string }[],
): ProblemDetails {
    return {
        type: 'about:blank',
        title: code,
        status,
        detail: code,
        code,
        ...(errors && {
            errors: errors.map((e) => ({ message: e.code, ...e })),
        }),
    };
}

/** Called from `test/setup.ts` after every test. */
export function takeUnhandled(): readonly string[] {
    const found = active?.unhandled ?? [];
    active = undefined;
    return found;
}
