import { screen, waitFor } from '@testing-library/react';
import { Route, Routes } from 'react-router-dom';
import { expect, test } from 'vitest';
import { makeMe, mockApi, problem, renderApp } from '../../../../test/utils';
import { NewTopicPage } from './NewTopicPage';

const BOARD = '/api/v1/boards/{id}' as const;
const CREATE = '/api/v1/boards/{id}/topics' as const;

function setup() {
    const mock = mockApi();
    mock.on('get', BOARD, 200, {
        board: {
            id: 'board-1',
            name: 'General',
            description: '',
            categoryId: 'c',
            categoryName: 'First category',
            adminOnly: false,
            topicCount: 0,
            postCount: 0,
        },
        topics: { items: [], page: 1, perPage: 20, total: 0, totalPages: 0 },
    });
    const view = renderApp(
        <Routes>
            <Route
                path="forum/boards/:boardId/new-topic"
                element={<NewTopicPage />}
            />
            <Route
                path="forum/topics/:topicId"
                element={<p>Topic created</p>}
            />
        </Routes>,
        { mock, me: makeMe(), route: '/forum/boards/board-1/new-topic' },
    );
    return view;
}

test('validates title and body client side without calling the API', async () => {
    const { user, mock } = setup();
    await user.click(await screen.findByRole('button', { name: 'Post topic' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
        '2 errors need your attention',
    );
    expect(mock.calls('post', CREATE)).toHaveLength(0);
    await user.type(screen.getByLabelText(/Title/), 'ab');
    await user.type(screen.getByRole('textbox', { name: /Message/ }), 'x');
    await user.click(screen.getByRole('button', { name: 'Post topic' }));
    expect(
        (await screen.findAllByText('This is too short.')).length,
    ).toBeGreaterThan(0);
});

test('creates the topic and opens it', async () => {
    const { user, mock } = setup();
    mock.on('post', CREATE, 201, { id: 'new-topic' });
    await user.type(await screen.findByLabelText(/Title/), 'My topic');
    await user.type(
        screen.getByRole('textbox', { name: /Message/ }),
        'Body text',
    );
    await user.click(screen.getByRole('button', { name: 'Post topic' }));
    expect(await screen.findByText('Topic created')).toBeInTheDocument();
    expect(mock.calls('post', CREATE)[0].body).toEqual({
        title: 'My topic',
        body: 'Body text',
    });
});

test('shows server errors and keeps the typed text', async () => {
    const { user, mock } = setup();
    mock.onUnlisted('post', CREATE, 429, problem(429, 'posting_too_fast'));
    const title = await screen.findByLabelText(/Title/);
    await user.type(title, 'My topic');
    await user.type(
        screen.getByRole('textbox', { name: /Message/ }),
        'Body text',
    );
    await user.click(screen.getByRole('button', { name: 'Post topic' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
        'posting too fast',
    );
    expect(title).toHaveValue('My topic');
    await waitFor(() =>
        expect(
            screen.getByRole('button', { name: 'Post topic' }),
        ).toBeEnabled(),
    );
});

test('maps API field errors onto the field', async () => {
    const { user, mock } = setup();
    mock.on(
        'post',
        CREATE,
        422,
        problem(422, 'validation_failed', [
            { field: 'title', code: 'invalid_characters' },
        ]),
    );
    await user.type(await screen.findByLabelText(/Title/), 'My topic');
    await user.type(
        screen.getByRole('textbox', { name: /Message/ }),
        'Body text',
    );
    await user.click(screen.getByRole('button', { name: 'Post topic' }));
    expect(
        (
            await screen.findAllByText(
                'This contains characters that are not allowed.',
            )
        ).length,
    ).toBeGreaterThan(0);
    expect(screen.getByLabelText(/Title/)).toHaveAttribute(
        'aria-invalid',
        'true',
    );
});
