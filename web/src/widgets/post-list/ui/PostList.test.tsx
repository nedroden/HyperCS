import { screen, waitFor } from '@testing-library/react';
import { expect, test } from 'vitest';
import { ALICE_ID, makePost, makeTopic } from '../../../../test/forumFixtures';
import { queryKeys } from '@/shared/api';
import { makeMe, renderApp, type Me } from '../../../../test/utils';
import { PostList } from './PostList';

const own = makePost({
    id: 'post-1',
    author: {
        id: ALICE_ID,
        username: 'alice',
        displayName: 'Alice',
        role: 'member',
        joinedAt: '2025-12-01T00:00:00Z',
        postCount: 2,
    },
    editedAt: '2026-01-02T00:00:00Z',
});
const other = makePost({ id: 'post-2', number: 2 });

function renderList(me: Me | null, locked = false) {
    return renderApp(
        <PostList
            topic={makeTopic({ locked })}
            posts={[own, other]}
            onQuote={() => undefined}
        />,
        { me },
    );
}

/** Waits until `/me` has been answered, so "absent" assertions do not race the session. */
const sessionReady = (rendered: ReturnType<typeof renderList>) =>
    waitFor(() =>
        expect(
            rendered.queryClient.getQueryState(queryKeys.me.all)?.status,
        ).toBe('success'),
    );

const links = (name: RegExp) => screen.queryAllByRole('link', { name });
const buttons = (name: RegExp) => screen.queryAllByRole('button', { name });

test('guests see no actions', async () => {
    await sessionReady(renderList(null));
    expect(await screen.findAllByRole('article')).toHaveLength(2);
    expect(buttons(/quote|delete/i)).toHaveLength(0);
    expect(links(/^edit$/i)).toHaveLength(0);
});

test('a member gets quote on all posts but edit/delete/history only on their own', async () => {
    renderList(makeMe({ id: ALICE_ID }));
    expect(
        await screen.findAllByRole('button', { name: 'Quote' }),
    ).toHaveLength(2);
    expect(links(/^Edit$/)).toHaveLength(1);
    expect(buttons(/^Delete$/)).toHaveLength(1);
    expect(links(/edit history/i)).toHaveLength(1);
});

test('a member who is not the author cannot edit or delete', async () => {
    renderList(makeMe({ id: 'someone-else' }));
    expect(
        await screen.findAllByRole('button', { name: 'Quote' }),
    ).toHaveLength(2);
    expect(links(/^Edit$/)).toHaveLength(0);
    expect(buttons(/^Delete$/)).toHaveLength(0);
    expect(links(/edit history/i)).toHaveLength(0);
});

test.each(['moderator', 'admin'] as const)(
    'a %s can edit and delete every post',
    async (role) => {
        renderList(makeMe({ id: 'staff', role }));
        expect(
            await screen.findAllByRole('link', { name: 'Edit' }),
        ).toHaveLength(2);
        expect(buttons(/^Delete$/)).toHaveLength(2);
    },
);

test('in a locked topic members cannot quote but moderators can', async () => {
    const first = renderList(makeMe({ id: ALICE_ID }), true);
    await sessionReady(first);
    await screen.findAllByRole('article');
    expect(buttons(/^Quote$/)).toHaveLength(0);
    first.unmount();
    renderList(makeMe({ id: 'mod', role: 'moderator' }), true);
    expect(
        await screen.findAllByRole('button', { name: 'Quote' }),
    ).toHaveLength(2);
});

test('renders Markdown, the edited marker and Re: titles', async () => {
    renderList(null);
    expect((await screen.findAllByText('world'))[0].tagName).toBe('STRONG');
    expect(screen.getByText('Welcome')).toBeInTheDocument();
    expect(screen.getByText('Re: Welcome')).toBeInTheDocument();
});
