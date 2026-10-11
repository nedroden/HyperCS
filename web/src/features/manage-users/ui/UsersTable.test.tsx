import { screen, waitFor, within } from '@testing-library/react';
import { expect, test } from 'vitest';
import type { Schema } from '@/shared/api';
import { makeMe, mockApi, problem, renderApp } from '../../../../test/utils';
import { UsersTable } from './UsersTable';

const ME = makeMe({
    id: '0190c3a2-0000-7000-8000-0000000000aa',
    username: 'admin',
    displayName: 'Administrator',
    role: 'admin',
});

function user(overrides: Partial<Schema<'AdminUser'>>): Schema<'AdminUser'> {
    return {
        id: 'u',
        username: 'u',
        displayName: 'U',
        email: 'u@example.com',
        role: 'member',
        postCount: 0,
        createdAt: new Date(Date.now() - 5 * 86_400_000).toISOString(),
        ...overrides,
    };
}

const ITEMS = [
    user({
        id: ME.id,
        username: 'admin',
        displayName: 'Administrator',
        role: 'admin',
    }),
    user({ id: 'u2', username: 'jane', displayName: 'Jane' }),
];

function setup(totalPages = 1) {
    const mock = mockApi();
    mock.on('get', '/api/v1/users', 200, {
        items: ITEMS,
        page: 1,
        perPage: 20,
        total: 2,
        totalPages,
    });
    return renderApp(<UsersTable />, { mock, me: ME });
}

test('lists users with a role dropdown; the own row is locked', async () => {
    setup();
    const jane = await screen.findByRole('combobox', { name: 'Role of jane' });
    expect(jane).toHaveValue('member');
    expect(jane).toBeEnabled();
    expect(
        within(jane)
            .getAllByRole('option')
            .map((o) => o.textContent),
    ).toEqual(['Admin', 'Moderator', 'Member']);
    const own = screen.getByRole('combobox', { name: 'Role of admin' });
    expect(own).toBeDisabled();
    expect(own).toHaveAccessibleDescription('You cannot change your own role.');
    expect(screen.getAllByText('5 days ago')).toHaveLength(2);
});

test('changes a role through PATCH', async () => {
    const { mock, user: ui } = setup();
    mock.on('patch', '/api/v1/users/{username}/role', 204);
    await ui.selectOptions(
        await screen.findByRole('combobox', { name: 'Role of jane' }),
        'moderator',
    );
    await waitFor(() =>
        expect(
            mock.calls('patch', '/api/v1/users/{username}/role'),
        ).toHaveLength(1),
    );
    const call = mock.calls('patch', '/api/v1/users/{username}/role')[0];
    expect(call.params.username).toBe('jane');
    expect(call.body).toEqual({ role: 'moderator' });
    expect(await screen.findByRole('status')).toHaveTextContent(
        'The role of Jane was changed to Moderator.',
    );
});

test('shows the last_admin conflict and keeps the old role', async () => {
    const { mock, user: ui } = setup();
    mock.on(
        'patch',
        '/api/v1/users/{username}/role',
        409,
        problem(409, 'last_admin'),
    );
    const select = await screen.findByRole('combobox', {
        name: 'Role of jane',
    });
    await ui.selectOptions(select, 'admin');
    expect(await screen.findByRole('alert')).toHaveTextContent(
        'The role of Jane could not be changed. The last administrator cannot be demoted.',
    );
    await waitFor(() => expect(select).toHaveValue('member'));
});

test('paginates with ?page= links', async () => {
    setup(3);
    expect(await screen.findByRole('link', { name: 'Page 2' })).toHaveAttribute(
        'href',
        '/?page=2',
    );
});
