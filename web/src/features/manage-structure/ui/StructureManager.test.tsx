import { screen, waitFor, within } from '@testing-library/react';
import { expect, test } from 'vitest';
import type { Schema } from '@/shared/api';
import { mockApi, problem, renderApp } from '../../../../test/utils';
import { useCategories } from '@/entities/category';
import { StructureManager } from './StructureManager';

function board(id: string, name: string, categoryId: string, position: number) {
    return {
        id,
        categoryId,
        name,
        description: '',
        adminOnly: false,
        position,
        topicCount: 0,
        postCount: 0,
    };
}

const CATEGORIES: Schema<'CategoryWithBoards'>[] = [
    {
        id: 'c1',
        name: 'Administration',
        description: '',
        adminOnly: true,
        position: 0,
        boards: [board('b1', 'Forum administration', 'c1', 0)],
    },
    {
        id: 'c2',
        name: 'My first category',
        description: '',
        adminOnly: false,
        position: 1,
        boards: [
            board('b2', 'General Discussion', 'c2', 0),
            board('b3', 'Second board', 'c2', 1),
        ],
    },
];

function setup() {
    const mock = mockApi();
    mock.on('get', '/api/v1/categories', 200, { categories: CATEGORIES });
    return renderApp(<StructureManager />, { mock });
}

test('lists categories and boards with the admin-only badge and edge-aware reorder buttons', async () => {
    setup();
    expect(await screen.findByText('My first category')).toBeInTheDocument();
    expect(screen.getByText('Admin only')).toBeInTheDocument();
    expect(
        screen.getByRole('button', { name: 'Move category Administration up' }),
    ).toBeDisabled();
    expect(
        screen.getByRole('button', {
            name: 'Move category My first category down',
        }),
    ).toBeDisabled();
    expect(
        screen.getByRole('button', {
            name: 'Move board General Discussion up',
        }),
    ).toBeDisabled();
    expect(
        screen.getByRole('button', { name: 'Move board Second board up' }),
    ).toBeEnabled();
});

test('reorders a board and a category with the direction in the request', async () => {
    const { mock, user } = setup();
    mock.on('post', '/api/v1/boards/{id}/reorder', 204);
    mock.on('post', '/api/v1/categories/{id}/reorder', 204);
    await user.click(
        await screen.findByRole('button', {
            name: 'Move board Second board up',
        }),
    );
    await waitFor(() =>
        expect(mock.calls('post', '/api/v1/boards/{id}/reorder')).toHaveLength(
            1,
        ),
    );
    const boardCall = mock.calls('post', '/api/v1/boards/{id}/reorder')[0];
    expect(boardCall.params.id).toBe('b3');
    expect(boardCall.body).toEqual({ direction: 'up' });

    await user.click(
        screen.getByRole('button', {
            name: 'Move category Administration down',
        }),
    );
    await waitFor(() =>
        expect(
            mock.calls('post', '/api/v1/categories/{id}/reorder'),
        ).toHaveLength(1),
    );
    expect(
        mock.calls('post', '/api/v1/categories/{id}/reorder')[0].body,
    ).toEqual({ direction: 'down' });
});

test('creates a category and a board, validating the name first', async () => {
    const { mock, user } = setup();
    mock.on('post', '/api/v1/categories', 201, { id: 'c3' });
    mock.on('post', '/api/v1/boards', 201, { id: 'b4' });

    await user.click(
        await screen.findByRole('button', { name: 'Add category' }),
    );
    const dialog = screen.getByRole('dialog', { name: 'New category' });
    await user.click(within(dialog).getByRole('button', { name: 'Save' }));
    expect(await within(dialog).findByRole('alert')).toHaveTextContent(
        'This field is required.',
    );
    expect(mock.calls('post', '/api/v1/categories')).toHaveLength(0);

    await user.type(within(dialog).getByLabelText('Name'), 'News');
    await user.click(within(dialog).getByLabelText(/Admin only/));
    await user.click(within(dialog).getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(mock.calls('post', '/api/v1/categories')[0].body).toEqual({
        name: 'News',
        description: '',
        adminOnly: true,
    });

    await user.click(
        screen.getByRole('button', {
            name: 'Add board to My first category',
        }),
    );
    const boardDialog = screen.getByRole('dialog', { name: 'New board' });
    await user.type(within(boardDialog).getByLabelText('Name'), 'Off topic');
    await user.click(within(boardDialog).getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(mock.calls('post', '/api/v1/boards')[0].body).toEqual({
        categoryId: 'c2',
        name: 'Off topic',
        description: '',
        adminOnly: false,
    });
});

test('edits a category through PATCH and shows server field errors', async () => {
    const { mock, user } = setup();
    mock.on(
        'patch',
        '/api/v1/categories/{id}',
        422,
        problem(422, 'validation_failed', [
            { field: 'name', code: 'too_long' },
        ]),
    );
    await user.click(
        await screen.findByRole('button', { name: 'Edit My first category' }),
    );
    const dialog = screen.getByRole('dialog', { name: 'Edit category' });
    const name = within(dialog).getByLabelText('Name');
    expect(name).toHaveValue('My first category');
    await user.clear(name);
    await user.type(name, 'Renamed');
    await user.click(within(dialog).getByRole('button', { name: 'Save' }));
    expect(await within(dialog).findByRole('alert')).toHaveTextContent(
        'This is too long.',
    );
    expect(mock.calls('patch', '/api/v1/categories/{id}')[0].params.id).toBe(
        'c2',
    );
});

test('deleting a category asks for confirmation that explains the cascade', async () => {
    const { mock, user } = setup();
    mock.on('delete', '/api/v1/categories/{id}', 204);
    await user.click(
        await screen.findByRole('button', { name: 'Delete My first category' }),
    );
    const dialog = screen.getByRole('dialog', {
        name: 'Delete category “My first category”?',
    });
    expect(dialog).toHaveTextContent('its 2 boards and all topics and posts');
    // Cancel keeps everything.
    await user.click(within(dialog).getByRole('button', { name: 'Cancel' }));
    expect(screen.queryByRole('dialog')).toBeNull();
    expect(mock.calls('delete', '/api/v1/categories/{id}')).toHaveLength(0);

    await user.click(
        screen.getByRole('button', { name: 'Delete My first category' }),
    );
    await user.click(
        screen.getByRole('button', { name: 'Delete permanently' }),
    );
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(mock.calls('delete', '/api/v1/categories/{id}')[0].params.id).toBe(
        'c2',
    );
});

test('deleting a board shows API errors inside the dialog', async () => {
    const { mock, user } = setup();
    mock.on('delete', '/api/v1/boards/{id}', 404, problem(404, 'not_found'));
    await user.click(
        await screen.findByRole('button', { name: 'Delete Second board' }),
    );
    await user.click(
        screen.getByRole('button', { name: 'Delete permanently' }),
    );
    expect(await screen.findByRole('alert')).toHaveTextContent(
        'The requested item could not be found.',
    );
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(mock.calls('delete', '/api/v1/boards/{id}')[0].params.id).toBe('b3');
});

function BoardIndexProbe() {
    const index = useCategories();
    return <p>{index.data?.categories.length} categories in the board index</p>;
}

test('shares the categories cache entry with the board index without crashing (regression)', async () => {
    const mock = mockApi();
    mock.on('get', '/api/v1/categories', 200, { categories: CATEGORIES });
    renderApp(
        <>
            <BoardIndexProbe />
            <StructureManager />
        </>,
        { mock },
    );
    expect(
        await screen.findByText('2 categories in the board index'),
    ).toBeInTheDocument();
    expect(await screen.findByText('My first category')).toBeInTheDocument();
});
