import { screen } from '@testing-library/react';
import { expect, test } from 'vitest';
import { makeMe, mockApi, renderApp } from '../../test/utils';
import { AppRoutes } from './App';

test.each([
    '/admin',
    '/admin/settings',
    '/admin/structure',
    '/admin/users',
    '/admin/homepage',
    '/admin/news',
])(
    '%s is denied to members and the admin API is never called',
    async (route) => {
        const mock = mockApi();
        renderApp(<AppRoutes />, { route, me: makeMe(), mock });
        expect(await screen.findByText('Access denied')).toBeInTheDocument();
        expect(
            screen.queryByRole('navigation', { name: 'Administration' }),
        ).toBeNull();
        // Only the session and the public site settings (the layout shows the site title).
        expect(mock.allCalls.map((c) => c.path)).toEqual([
            '/api/v1/me',
            '/api/v1/cms/site',
        ]);
    },
);

test('admins reach the admin screens and get the sidebar', async () => {
    const mock = mockApi();
    mock.on('get', '/api/v1/categories', 200, { categories: [] });
    renderApp(<AppRoutes />, {
        route: '/admin/structure',
        me: makeMe({ role: 'admin' }),
        mock,
    });
    expect(
        await screen.findByRole('heading', { name: 'Admin · Forum structure' }),
    ).toBeInTheDocument();
    const tabs = screen.getByRole('navigation', { name: 'Administration' });
    expect(
        Array.from(tabs.querySelectorAll('a')).map((a) => a.textContent),
    ).toEqual([
        'Overview',
        'General',
        'Structure',
        'Users & roles',
        'Homepage',
        'News',
    ]);
    expect(screen.getByRole('link', { name: 'Structure' })).toHaveAttribute(
        'aria-current',
        'page',
    );
});

test('guests are sent to the login page', async () => {
    renderApp(<AppRoutes />, { route: '/admin/users' });
    expect(
        await screen.findByRole('heading', { name: 'Login' }),
    ).toBeInTheDocument();
});
