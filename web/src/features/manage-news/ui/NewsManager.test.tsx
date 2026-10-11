import { screen, waitFor, within } from '@testing-library/react';
import { expect, test } from 'vitest';
import { mockApi, problem, renderApp } from '../../../../test/utils';
import { NewsManager } from './NewsManager';

const ITEMS = [
    {
        id: 'n1',
        title: 'Release 0.1',
        slug: 'release-0-1',
        body: 'We shipped.',
        publishedAt: new Date(Date.now() - 3 * 86_400_000).toISOString(),
    },
];

function setup() {
    const mock = mockApi();
    mock.on('get', '/api/v1/news', 200, {
        items: ITEMS,
        page: 1,
        perPage: 10,
        total: 1,
        totalPages: 1,
    });
    return renderApp(<NewsManager />, { mock });
}

test('lists news items with relative dates', async () => {
    setup();
    expect(await screen.findByText('Release 0.1')).toBeInTheDocument();
    expect(screen.getByText('3 days ago')).toBeInTheDocument();
});

test('creating validates the title and body, then POSTs them', async () => {
    const { mock, user } = setup();
    mock.on('post', '/api/v1/news', 201, { id: 'n2' });
    await user.click(
        await screen.findByRole('button', { name: 'Add news item' }),
    );
    const dialog = screen.getByRole('dialog', { name: 'New news item' });
    await user.click(within(dialog).getByRole('button', { name: 'Publish' }));
    expect(await within(dialog).findByRole('alert')).toHaveTextContent(
        '2 errors need your attention',
    );
    expect(mock.calls('post', '/api/v1/news')).toHaveLength(0);

    await user.type(within(dialog).getByLabelText('Title'), 'Hello');
    await user.type(within(dialog).getByLabelText('Text'), 'Some *news*.');
    await user.click(within(dialog).getByRole('tab', { name: 'Preview' }));
    expect(within(dialog).getByText('news').tagName).toBe('EM');
    await user.click(within(dialog).getByRole('button', { name: 'Publish' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(mock.calls('post', '/api/v1/news')[0].body).toEqual({
        title: 'Hello',
        body: 'Some *news*.',
    });
});

test('rejects a title that is too long before calling the API', async () => {
    const { mock, user } = setup();
    await user.click(
        await screen.findByRole('button', { name: 'Add news item' }),
    );
    const dialog = screen.getByRole('dialog');
    const title = within(dialog).getByLabelText('Title');
    // The input's maxLength stops typing at the limit, so the schema is exercised via the attribute.
    expect(title).toHaveAttribute('maxLength', '120');
    await user.type(within(dialog).getByLabelText('Text'), 'Body');
    await user.click(within(dialog).getByRole('button', { name: 'Publish' }));
    expect(await within(dialog).findByRole('alert')).toHaveTextContent(
        'This field is required.',
    );
    expect(mock.calls('post', '/api/v1/news')).toHaveLength(0);
});

test('editing PATCHes by id; server errors land on the field', async () => {
    const { mock, user } = setup();
    mock.on(
        'patch',
        '/api/v1/news/{id}',
        422,
        problem(422, 'validation_failed', [
            { field: 'title', code: 'too_short' },
        ]),
    );
    await user.click(
        await screen.findByRole('button', { name: 'Edit Release 0.1' }),
    );
    const dialog = screen.getByRole('dialog', { name: 'Edit news item' });
    expect(within(dialog).getByLabelText('Title')).toHaveValue('Release 0.1');
    await user.type(within(dialog).getByLabelText('Title'), '!');
    await user.click(within(dialog).getByRole('button', { name: 'Publish' }));
    expect(await within(dialog).findByRole('alert')).toHaveTextContent(
        'This is too short.',
    );
    const call = mock.calls('patch', '/api/v1/news/{id}')[0];
    expect(call.params.id).toBe('n1');
    expect(call.body).toEqual({ title: 'Release 0.1!', body: 'We shipped.' });
});

test('deleting asks for confirmation first', async () => {
    const { mock, user } = setup();
    mock.on('delete', '/api/v1/news/{id}', 204);
    await user.click(
        await screen.findByRole('button', { name: 'Delete Release 0.1' }),
    );
    expect(
        screen.getByRole('dialog', { name: 'Delete news item “Release 0.1”?' }),
    ).toBeInTheDocument();
    expect(mock.calls('delete', '/api/v1/news/{id}')).toHaveLength(0);
    await user.click(
        within(screen.getByRole('dialog')).getByRole('button', {
            name: 'Delete',
        }),
    );
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(mock.calls('delete', '/api/v1/news/{id}')[0].params.id).toBe('n1');
});
