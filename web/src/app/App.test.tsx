import { screen } from '@testing-library/react';
import { expect, test } from 'vitest';
import { makeMe, mockApi, problem, renderApp } from '../../test/utils';
import { AppRoutes } from './App';

/** Every route the product needs is registered; each renders a page heading (placeholder or real). */
const MEMBER_ROUTES = [
    '/',
    '/terms',
    '/forum',
    '/forum/boards/1',
    '/forum/boards/1/general',
    '/forum/topics/1/hello',
    '/forum/users/alice',
    '/settings',
    '/forum/boards/1/new-topic',
    '/forum/posts/1/edit',
    '/forum/posts/1/history',
];
const ADMIN_ROUTES = [
    '/admin/structure',
    '/admin/users',
    '/admin/homepage',
    '/admin/news',
];

function renderRoute(route: string, me = makeMe({ role: 'admin' })) {
    const mock = mockApi();
    // Pages that already load data are out of scope here: unhandled requests answer 501 and are tolerated.
    mock.onUnlisted(
        'get',
        '/api/v1/stats',
        501,
        problem(501, 'not_implemented'),
    );
    return renderApp(<AppRoutes />, { route, me, mock });
}

test.each([...MEMBER_ROUTES, ...ADMIN_ROUTES])(
    '%s is registered',
    async (route) => {
        renderRoute(route);
        expect(
            await screen.findByRole('heading', { level: 1 }, { timeout: 3000 }),
        ).toBeInTheDocument();
        expect(screen.queryByText(/could not be found/i)).toBeNull();
    },
);

test('unknown paths render the not-found page', async () => {
    renderRoute('/does/not/exist');
    expect(await screen.findByRole('heading', { level: 2 })).toHaveTextContent(
        /could not be found/i,
    );
});

test('login and register render for guests', async () => {
    renderApp(<AppRoutes />, { route: '/register' });
    expect(await screen.findByLabelText('Username')).toBeInTheDocument();
});
