import { screen, waitFor, within } from '@testing-library/react';
import { Route, Routes } from 'react-router-dom';
import { expect, test } from 'vitest';
import {
    ALICE_ID,
    makePost,
    makeTopicPage,
} from '../../../../test/forumFixtures';
import { makeMe, mockApi, problem, renderApp } from '../../../../test/utils';
import { TopicPage } from './TopicPage';

const TOPIC = '/api/v1/topics/{id}' as const;

function renderTopic(
    options: { me?: ReturnType<typeof makeMe> | null; route?: string } = {},
    mock = mockApi(),
) {
    return renderApp(
        <Routes>
            <Route
                path="forum/topics/:topicId/:slug?"
                element={<TopicPage />}
            />
            <Route path="forum/boards/:boardId" element={<p>Board page</p>} />
        </Routes>,
        {
            mock,
            me: options.me ?? null,
            route: options.route ?? '/forum/topics/topic-1/welcome',
        },
    );
}

test('guest: shows posts, badges and a login prompt instead of the reply form', async () => {
    const mock = mockApi();
    mock.on(
        'get',
        TOPIC,
        200,
        makeTopicPage({ pinned: true, locked: true, replyCount: 1 }),
    );
    renderTopic({}, mock);
    expect(
        await screen.findByRole('heading', { name: /Welcome/ }),
    ).toBeInTheDocument();
    expect(screen.getByText('Pinned')).toBeInTheDocument();
    expect(screen.getAllByText('Locked').length).toBeGreaterThan(0);
    expect(screen.getByText(/2 posts/)).toBeInTheDocument();
    const reply = document.getElementById('reply') as HTMLElement;
    expect(
        within(reply).getByRole('link', { name: 'Log in' }),
    ).toBeInTheDocument();
    expect(within(reply).queryByRole('textbox')).toBeNull();
    expect(screen.queryByRole('group', { name: 'Moderation' })).toBeNull();
});

test('locked topic: member sees a locked notice, moderator still gets the form and the moderation bar', async () => {
    const mock = mockApi();
    mock.on('get', TOPIC, 200, makeTopicPage({ locked: true }));
    const member = renderTopic({ me: makeMe() }, mock);
    expect(
        await screen.findByText(/This topic is locked\. Only moderators/),
    ).toBeInTheDocument();
    expect(screen.queryByRole('group', { name: 'Moderation' })).toBeNull();
    member.unmount();

    renderTopic({ me: makeMe({ role: 'moderator' }) }, mock);
    expect(
        await screen.findByRole('group', { name: 'Moderation' }),
    ).toBeInTheDocument();
    expect(
        await screen.findByRole('button', { name: 'Unlock topic' }),
    ).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Submit' })).toBeInTheDocument();
});

test('paginates posts with ?page= and requests that page', async () => {
    const mock = mockApi();
    mock.on(
        'get',
        TOPIC,
        200,
        makeTopicPage({ replyCount: 20 }, undefined, {
            total: 21,
            totalPages: 2,
            page: 2,
        }),
    );
    renderTopic({ route: '/forum/topics/topic-1/welcome?page=2' }, mock);
    const nav = await screen.findByRole('navigation', { name: 'Pagination' });
    expect(within(nav).getByRole('link', { name: 'Page 1' })).toHaveAttribute(
        'href',
        expect.stringContaining('page=1'),
    );
    expect(mock.calls('get', TOPIC)[0].search.get('page')).toBe('2');
});

test('quote inserts a Markdown blockquote into the reply form, and replying posts the draft', async () => {
    const mock = mockApi();
    mock.on('get', TOPIC, 200, makeTopicPage());
    mock.on('post', '/api/v1/topics/{id}/posts', 201, { id: 'post-3' });
    const { user } = renderTopic({ me: makeMe() }, mock);
    await user.click(
        (await screen.findAllByRole('button', { name: 'Quote' }))[1],
    );
    const box = screen.getByRole('textbox', { name: /Message/ });
    expect(box).toHaveValue('> **Carol**:\n>\n> A reply\n\n');
    await user.type(box, 'I agree');
    await user.click(screen.getByRole('button', { name: 'Submit' }));
    await waitFor(() =>
        expect(mock.calls('post', '/api/v1/topics/{id}/posts')).toHaveLength(1),
    );
    expect(mock.calls('post', '/api/v1/topics/{id}/posts')[0].body).toEqual({
        body: '> **Carol**:\n>\n> A reply\n\nI agree',
    });
});

test.each([
    [429, 'posting_too_fast', 'You are posting too fast'],
    [409, 'duplicate_post', 'You already posted this.'],
] as const)(
    'reply error %s %s is translated and the draft is kept',
    async (status, code, text) => {
        const mock = mockApi();
        mock.on('get', TOPIC, 200, makeTopicPage());
        mock.onUnlisted(
            'post',
            '/api/v1/topics/{id}/posts',
            status,
            problem(status, code),
        );
        const { user } = renderTopic({ me: makeMe() }, mock);
        const box = await screen.findByRole('textbox', { name: /Message/ });
        await user.type(box, 'my draft');
        await user.click(screen.getByRole('button', { name: 'Submit' }));
        expect(await screen.findByRole('alert')).toHaveTextContent(text);
        expect(box).toHaveValue('my draft');
    },
);

test('deleting the first post warns that the topic is deleted and returns to the board', async () => {
    const mock = mockApi();
    const own = makePost({
        author: {
            ...makePost().author,
            id: ALICE_ID,
            username: 'alice',
            displayName: 'Alice',
        },
    });
    mock.on('get', TOPIC, 200, makeTopicPage({}, [own]));
    mock.on('delete', '/api/v1/posts/{id}', 204);
    const { user } = renderTopic({ me: makeMe() }, mock);
    await user.click(await screen.findByRole('button', { name: 'Delete' }));
    const dialog = await screen.findByRole('dialog');
    expect(dialog).toHaveTextContent('Deleting it deletes the whole topic');
    await user.click(within(dialog).getByRole('button', { name: 'Delete' }));
    expect(await screen.findByText('Board page')).toBeInTheDocument();
    expect(mock.calls('delete', '/api/v1/posts/{id}')[0].params.id).toBe(
        'post-1',
    );
});

test('moderator can pin a topic', async () => {
    const mock = mockApi();
    mock.on('get', TOPIC, 200, makeTopicPage());
    mock.on('patch', TOPIC, 204);
    const { user } = renderTopic({ me: makeMe({ role: 'admin' }) }, mock);
    await user.click(await screen.findByRole('button', { name: 'Pin topic' }));
    await waitFor(() => expect(mock.calls('patch', TOPIC)).toHaveLength(1));
    expect(mock.calls('patch', TOPIC)[0].body).toEqual({ pinned: true });
});

test('moves a topic to another board', async () => {
    const mock = mockApi();
    mock.on('get', TOPIC, 200, makeTopicPage());
    mock.on('patch', TOPIC, 204);
    const board = (id: string, name: string) => ({
        id,
        name,
        description: '',
        adminOnly: false,
        categoryId: 'cat-1',
        position: 0,
        postCount: 0,
        topicCount: 0,
    });
    mock.on('get', '/api/v1/categories', 200, {
        categories: [
            {
                id: 'cat-1',
                name: 'First category',
                description: '',
                adminOnly: false,
                position: 0,
                boards: [
                    board('board-1', 'General'),
                    board('board-2', 'Second board'),
                ],
            },
        ],
    });
    const { user } = renderTopic({ me: makeMe({ role: 'moderator' }) }, mock);
    await user.click(await screen.findByRole('button', { name: 'Move topic' }));
    const dialog = await screen.findByRole('dialog');
    const select = await within(dialog).findByLabelText('Destination board');
    expect(
        within(select).queryByRole('option', { name: 'General' }),
    ).toBeNull();
    await user.click(
        within(dialog).getByRole('button', { name: 'Move topic' }),
    );
    await waitFor(() => expect(mock.calls('patch', TOPIC)).toHaveLength(1));
    expect(mock.calls('patch', TOPIC)[0].body).toEqual({ boardId: 'board-2' });
});

test('unknown topic shows the not-found state', async () => {
    const mock = mockApi();
    mock.on('get', TOPIC, 404, problem(404, 'not_found'));
    renderTopic({}, mock);
    expect(await screen.findByRole('alert')).toHaveTextContent('Not found');
});
