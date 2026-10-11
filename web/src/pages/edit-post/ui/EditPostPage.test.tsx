import { screen } from '@testing-library/react';
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
import { EditPostPage } from './EditPostPage';

const PATCH = '/api/v1/posts/{id}' as const;

const mine = makePost({
    body: 'Original',
    author: {
        ...makePost().author,
        id: ALICE_ID,
        username: 'alice',
        displayName: 'Alice',
    },
});

const GET = '/api/v1/posts/{id}' as const;

function setup(me: Me | null, notFound = false) {
    const mock = mockApi();
    if (notFound) mock.onUnlisted('get', GET, 404, problem(404, 'not_found'));
    else {
        mock.on(
            'get',
            GET,
            200,
            makePostDetail({ ...mine, editedAt: '2026-01-02T00:00:00Z' }),
        );
    }
    mock.on('get', '/api/v1/posts/{id}/revisions', 200, {
        items: [],
        page: 1,
        perPage: 1,
        total: 2,
        totalPages: 2,
    });
    const view = renderApp(
        <Routes>
            <Route path="forum/posts/:postId/edit" element={<EditPostPage />} />
            <Route
                path="forum/topics/:topicId/:slug?"
                element={<p>Topic page</p>}
            />
        </Routes>,
        {
            mock,
            me,
            route: '/forum/posts/post-1/edit',
        },
    );
    return view;
}

test('saves the edited body and returns to the topic', async () => {
    const { user, mock } = setup(makeMe({ id: ALICE_ID }));
    mock.on('patch', PATCH, 204);
    const box = await screen.findByRole('textbox', { name: /Message/ });
    expect(box).toHaveValue('Original');
    expect(
        await screen.findByRole('link', {
            name: 'View edit history (2 revisions)',
        }),
    ).toBeInTheDocument();
    await user.clear(box);
    await user.type(box, 'Changed');
    await user.click(screen.getByRole('button', { name: 'Save changes' }));
    expect(await screen.findByText('Topic page')).toBeInTheDocument();
    expect(mock.calls('patch', PATCH)[0].body).toEqual({ body: 'Changed' });
});

test('keeps the text when the server rejects the edit', async () => {
    const { user, mock } = setup(makeMe({ id: ALICE_ID }));
    mock.onUnlisted('patch', PATCH, 403, problem(403, 'forbidden'));
    const box = await screen.findByRole('textbox', { name: /Message/ });
    await user.type(box, '!');
    await user.click(screen.getByRole('button', { name: 'Save changes' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
        'You are not allowed to do that.',
    );
    expect(box).toHaveValue('Original!');
});

test("another member cannot open someone else's post for editing", async () => {
    setup(makeMe({ id: 'other' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('No access');
});

test('loads the post by id on a direct visit and shows the breadcrumbs', async () => {
    const { mock } = setup(makeMe({ id: ALICE_ID }));
    expect(await screen.findByRole('textbox', { name: /Message/ })).toHaveValue(
        'Original',
    );
    expect(mock.calls('get', GET)[0].params).toEqual({ id: 'post-1' });
    expect(screen.getByRole('link', { name: 'General' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Welcome' })).toBeInTheDocument();
});

test('shows not-found for an unknown post', async () => {
    setup(makeMe({ id: ALICE_ID }), true);
    expect(await screen.findByText('Not found')).toBeInTheDocument();
});
