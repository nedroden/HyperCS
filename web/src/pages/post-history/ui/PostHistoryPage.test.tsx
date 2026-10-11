import { screen, waitFor } from '@testing-library/react';
import { Route, Routes } from 'react-router-dom';
import { expect, test } from 'vitest';
import {
    ALICE_ID,
    makePost,
    makePostDetail,
} from '../../../../test/forumFixtures';
import {
    makeMe,
    mockApi,
    problem,
    renderApp,
    type Me,
} from '../../../../test/utils';
import { PostHistoryPage } from './PostHistoryPage';

const REVISIONS = '/api/v1/posts/{id}/revisions' as const;
const RESTORE = '/api/v1/posts/{id}/revisions/{revisionId}/restore' as const;

const post = makePost({
    id: 'post-1',
    body: 'Current body',
    editedAt: '2026-01-02T00:00:00Z',
});

const GET = '/api/v1/posts/{id}' as const;

function setup(me: Me | null, notFound = false) {
    const mock = mockApi();
    if (notFound) mock.onUnlisted('get', GET, 404, problem(404, 'not_found'));
    else mock.on('get', GET, 200, makePostDetail(post));
    mock.on('get', REVISIONS, 200, {
        items: [
            {
                id: 'rev-1',
                body: 'Old body',
                createdAt: '2026-01-02T00:00:00Z',
                editedBy: null,
            },
        ],
        page: 1,
        perPage: 20,
        total: 1,
        totalPages: 1,
    });
    const view = renderApp(
        <Routes>
            <Route
                path="forum/posts/:postId/history"
                element={<PostHistoryPage />}
            />
            <Route
                path="forum/topics/:topicId/:slug?"
                element={<p>Topic page</p>}
            />
        </Routes>,
        {
            mock,
            me,
            route: '/forum/posts/post-1/history',
        },
    );
    return view;
}

test('staff see revisions and can restore one, which goes back to the topic', async () => {
    const { user, mock } = setup(makeMe({ role: 'moderator' }));
    mock.on('post', RESTORE, 204);
    expect(await screen.findByText('Old body')).toBeInTheDocument();
    expect(screen.getByText('Current body')).toBeInTheDocument();
    await user.click(
        screen.getByRole('button', { name: 'Restore this version' }),
    );
    expect(await screen.findByText('Topic page')).toBeInTheDocument();
    expect(mock.calls('post', RESTORE)[0].params).toEqual({
        id: 'post-1',
        revisionId: 'rev-1',
    });
});

test('the author of the post may restore too', async () => {
    setup(makeMe({ id: post.author.id }));
    expect(
        await screen.findByRole('button', { name: 'Restore this version' }),
    ).toBeInTheDocument();
});

test('a member who is neither author nor staff gets a no-access state and no history request', async () => {
    const { mock } = setup(makeMe({ id: ALICE_ID }));
    expect(await screen.findByRole('alert')).toHaveTextContent('No access');
    await waitFor(() => expect(mock.calls('get', REVISIONS)).toHaveLength(0));
    expect(
        screen.queryByRole('button', { name: 'Restore this version' }),
    ).toBeNull();
});

test('loads the post by id on a direct visit and shows the breadcrumbs', async () => {
    const { mock } = setup(makeMe({ role: 'moderator' }));
    expect(await screen.findByText('Current body')).toBeInTheDocument();
    expect(mock.calls('get', GET)[0].params).toEqual({ id: 'post-1' });
    expect(screen.getByRole('link', { name: 'General' })).toBeInTheDocument();
});

test('shows not-found for an unknown post', async () => {
    setup(makeMe({ role: 'moderator' }), true);
    expect(await screen.findByText('Not found')).toBeInTheDocument();
});
