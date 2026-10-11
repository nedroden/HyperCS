import { screen } from '@testing-library/react';
import { expect, test } from 'vitest';
import { bob } from '../../../../test/forumFixtures';
import { makeMe, mockApi, problem, renderApp } from '../../../../test/utils';
import { ForumIndexPage } from './ForumIndexPage';

const stats = {
    topics: 8,
    posts: 151,
    members: 3,
    boards: 2,
    onlineCount: 1,
    onlineUsers: [bob],
    newestMember: bob,
    latestPost: {
        topicId: 't1',
        postId: 'p1',
        topicTitle: 'Latest topic',
        author: bob,
        createdAt: '2026-01-01T00:00:00Z',
    },
};
const board = (id: string, name: string, withLast: boolean) => ({
    id,
    name,
    description: `${name} desc`,
    adminOnly: false,
    categoryId: 'c1',
    position: 0,
    topicCount: withLast ? 1 : 0,
    postCount: withLast ? 5 : 0,
    lastPost: withLast
        ? {
              topicId: 't1',
              postId: 'p1',
              topicTitle: 'Latest topic',
              author: bob,
              createdAt: '2026-01-01T00:00:00Z',
          }
        : null,
});

function setup() {
    const mock = mockApi();
    mock.on('get', '/api/v1/categories', 200, {
        categories: [
            {
                id: 'c1',
                name: 'My first category',
                description: 'Cat desc',
                adminOnly: false,
                position: 0,
                boards: [
                    board('b1', 'General', true),
                    board('b2', 'Second', false),
                ],
            },
        ],
    });
    mock.on('get', '/api/v1/stats', 200, stats);
    return mock;
}

test('guest: guest banner, boards with counters and last post, statistics', async () => {
    renderApp(<ForumIndexPage />, { mock: setup() });
    expect(
        await screen.findByRole('link', { name: 'General' }),
    ).toHaveAttribute('href', '/forum/boards/b1');
    expect(screen.getByText('1 topic')).toBeInTheDocument();
    expect(screen.getByText('5 posts')).toBeInTheDocument();
    expect(screen.getByText('0 topics')).toBeInTheDocument();
    expect(
        (await screen.findAllByRole('link', { name: 'Latest topic' }))[0],
    ).toHaveAttribute('href', '/forum/topics/t1');
    expect(await screen.findByText('Statistics')).toBeInTheDocument();
    expect(screen.getByText('151')).toBeInTheDocument();
    expect(screen.getByText(/You are browsing as a guest/)).toBeInTheDocument();
});

test('member: no guest banner', async () => {
    renderApp(<ForumIndexPage />, { mock: setup(), me: makeMe() });
    await screen.findByRole('link', { name: 'General' });
    expect(screen.queryByText(/browsing as a guest/)).toBeNull();
});

test('shows an error state with retry when the index fails', async () => {
    const mock = mockApi();
    mock.onUnlisted(
        'get',
        '/api/v1/categories',
        500,
        problem(500, 'internal_error'),
    );
    mock.on('get', '/api/v1/stats', 200, stats);
    renderApp(<ForumIndexPage />, { mock });
    expect(
        await screen.findByRole('button', { name: 'Try again' }),
    ).toBeInTheDocument();
});
