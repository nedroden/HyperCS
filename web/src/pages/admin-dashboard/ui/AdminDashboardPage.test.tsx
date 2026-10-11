import { screen, within } from '@testing-library/react';
import { expect, test } from 'vitest';
import { makeMe, mockApi, problem, renderApp } from '../../../../test/utils';
import { AdminDashboardPage } from './AdminDashboardPage';

const STATS = {
    boards: 4,
    members: 12,
    onlineCount: 2,
    onlineUsers: [],
    posts: 345,
    topics: 67,
};

function setup() {
    const mock = mockApi();
    mock.on('get', '/api/v1/stats', 200, STATS);
    return renderApp(<AdminDashboardPage />, {
        mock,
        route: '/admin',
        me: makeMe({ role: 'admin' }),
    });
}

test('shows the community statistics', async () => {
    setup();
    await screen.findByText('Online now');
    const stats = screen.getByRole('region', { name: 'Statistics' });
    for (const [label, value] of [
        ['Members', '12'],
        ['Topics', '67'],
        ['Posts', '345'],
        ['Boards', '4'],
        ['Online now', '2'],
    ]) {
        expect(
            within(stats).getByText(label).previousSibling,
        ).toHaveTextContent(value);
    }
});

test('links to every admin section from large tiles', async () => {
    setup();
    const tiles = await screen.findByRole('region', { name: 'Manage' });
    const links = within(tiles).getAllByRole('link');
    expect(links.map((a) => a.getAttribute('href'))).toEqual([
        '/admin/settings',
        '/admin/structure',
        '/admin/users',
        '/admin/homepage',
        '/admin/news',
    ]);
    expect(
        within(tiles).getByRole('link', { name: /General/ }),
    ).toHaveTextContent('Site title, slogan and default language.');
});

test('marks the overview as current in the sidebar, with icons on every label', async () => {
    setup();
    const nav = await screen.findByRole('navigation', {
        name: 'Administration',
    });
    expect(within(nav).getByRole('link', { name: 'Overview' })).toHaveAttribute(
        'aria-current',
        'page',
    );
    expect(nav.querySelectorAll('a > svg')).toHaveLength(6);
});

test('a failing statistics request does not hide the section links', async () => {
    const mock = mockApi();
    mock.onUnlisted(
        'get',
        '/api/v1/stats',
        500,
        problem(500, 'internal_error'),
    );
    renderApp(<AdminDashboardPage />, { mock, me: makeMe({ role: 'admin' }) });
    const tiles = await screen.findByRole('region', { name: 'Manage' });
    expect(within(tiles).getAllByRole('link')).toHaveLength(5);
});
