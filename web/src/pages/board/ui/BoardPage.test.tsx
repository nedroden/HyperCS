import { screen, within } from '@testing-library/react';
import { Route, Routes } from 'react-router-dom';
import { expect, test } from 'vitest';
import { bob } from '../../../../test/forumFixtures';
import { makeMe, mockApi, problem, renderApp } from '../../../../test/utils';
import { BoardPage } from './BoardPage';

const BOARD = '/api/v1/boards/{id}' as const;

const board = {
    id: 'board-1',
    name: 'General',
    description: 'Talk',
    categoryId: 'cat-1',
    categoryName: 'First category',
    adminOnly: false,
    topicCount: 2,
    postCount: 3,
};
const topic = (id: string, title: string, pinned = false) => ({
    id,
    title,
    slug: title.toLowerCase(),
    author: bob,
    createdAt: '2026-01-01T00:00:00Z',
    pinned,
    locked: false,
    replyCount: 1,
    viewCount: 9,
    lastPost: { postId: 'p', author: bob, createdAt: '2026-01-02T00:00:00Z' },
});

test('lists topics with badges and slug links, paginated', async () => {
    const mock = mockApi();
    mock.on('get', BOARD, 200, {
        board,
        topics: {
            items: [topic('t1', 'Pinned one', true), topic('t2', 'Second')],
            page: 2,
            perPage: 20,
            total: 22,
            totalPages: 2,
        },
    });
    renderApp(
        <Routes>
            <Route path="forum/boards/:boardId" element={<BoardPage />} />
        </Routes>,
        { mock, route: '/forum/boards/board-1?page=2' },
    );
    const link = await screen.findByRole('link', { name: 'Pinned one' });
    expect(link).toHaveAttribute('href', '/forum/topics/t1/pinned%20one');
    expect(screen.getByText('Pinned')).toBeInTheDocument();
    expect(mock.calls('get', BOARD)[0].search.get('page')).toBe('2');
    const nav = screen.getByRole('navigation', { name: 'Pagination' });
    expect(
        within(nav).getByRole('link', { name: 'Page 1' }),
    ).toBeInTheDocument();
    // Guests get no "New topic" button.
    expect(screen.queryByRole('link', { name: 'New topic' })).toBeNull();
});

test('empty board offers a new topic to members; unknown board is not found', async () => {
    const mock = mockApi();
    mock.on('get', BOARD, 200, {
        board,
        topics: { items: [], page: 1, perPage: 20, total: 0, totalPages: 0 },
    });
    const ui = (
        <Routes>
            <Route path="forum/boards/:boardId" element={<BoardPage />} />
        </Routes>
    );
    const view = renderApp(ui, {
        mock,
        me: makeMe(),
        route: '/forum/boards/board-1',
    });
    expect(await screen.findByText('No topics yet')).toBeInTheDocument();
    expect(
        screen.getAllByRole('link', { name: 'New topic' }).length,
    ).toBeGreaterThan(0);
    view.unmount();

    const missing = mockApi();
    missing.on('get', BOARD, 404, problem(404, 'board_not_found'));
    renderApp(ui, { mock: missing, route: '/forum/boards/nope' });
    expect(await screen.findByRole('alert')).toHaveTextContent('Not found');
});
