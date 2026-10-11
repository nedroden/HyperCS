import { QueryClient } from '@tanstack/react-query';
import { render } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { ReactElement, ReactNode } from 'react';
import { MemoryRouter } from 'react-router-dom';
import { Providers } from '@/app/lib/Providers';
import type { Schema } from '@/shared/api';
import { mockApi, problem, type MockApi } from './mockApi';

export type Me = Schema<'MeResponse'>;

/** A complete `MeResponse` (a plain member); override what the test cares about. */
export function makeMe(overrides: Partial<Me> = {}): Me {
    return {
        id: '0190c3a2-0000-7000-8000-000000000001',
        username: 'alice',
        displayName: 'Alice',
        email: 'alice@example.com',
        role: 'member',
        language: 'en-US',
        timezone: 'UTC',
        csrfToken: 'csrf-token-1',
        createdAt: '2026-01-01T00:00:00Z',
        postCount: 0,
        notifyOnQuote: true,
        notifyOnReply: true,
        showOnline: true,
        ...overrides,
    };
}

type RenderAppOptions = {
    /** Initial URL (or location with router state) for the MemoryRouter. */
    route?: string | { pathname: string; state?: unknown };
    /** `null` (default) = guest (`/me` answers 401); a `Me` = logged in; `'error'` = `/me` fails with a 500. */
    me?: Me | null | 'error';
    /** Reuse a mock created earlier (e.g. to register routes before rendering). */
    mock?: MockApi;
};

/**
 * Renders `ui` with everything the real app has: a fresh QueryClient (no retries), a MemoryRouter, the
 * session provider and a mocked fetch boundary whose `/me` answers according to `options.me`.
 * `ui` is usually a component, or `<AppRoutes />`-like `<Routes>` for guard and navigation tests.
 * Returns the RTL result plus `mock` (register more routes, inspect calls), `queryClient` and a `user`
 * (userEvent) instance.
 */
export function renderApp(ui: ReactElement, options: RenderAppOptions = {}) {
    const { route = '/', me = null } = options;
    const mock = options.mock ?? mockApi();
    if (me === 'error')
        mock.onUnlisted(
            'get',
            '/api/v1/me',
            500,
            problem(500, 'internal_error'),
        );
    else if (me) mock.on('get', '/api/v1/me', 200, me);
    else mock.on('get', '/api/v1/me', 401, problem(401, 'unauthorized'));

    const queryClient = new QueryClient({
        defaultOptions: {
            queries: { retry: false, gcTime: Infinity },
            mutations: { retry: false },
        },
    });
    function TestRouter({ children }: { children: ReactNode }) {
        return <MemoryRouter initialEntries={[route]}>{children}</MemoryRouter>;
    }
    const result = render(
        <Providers queryClient={queryClient} Router={TestRouter}>
            {ui}
        </Providers>,
    );
    return { ...result, mock, queryClient, user: userEvent.setup() };
}

export { mockApi, problem, type MockApi };
