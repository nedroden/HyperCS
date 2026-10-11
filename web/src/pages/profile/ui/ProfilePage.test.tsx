import { screen } from '@testing-library/react';
import { Route, Routes } from 'react-router-dom';
import { expect, test } from 'vitest';
import { problem, renderApp } from '../../../../test/utils';
import { ProfilePage } from './ProfilePage';

function page(route = '/forum/users/jane') {
    return renderApp(
        <Routes>
            <Route path="/forum/users/:username" element={<ProfilePage />} />
        </Routes>,
        { route },
    );
}

const jane = {
    id: '0190c3a2-0000-7000-8000-000000000002',
    username: 'jane',
    displayName: 'Jane Doe',
    role: 'moderator' as const,
    createdAt: '2020-01-01T00:00:00Z',
    postCount: 4,
    location: 'Berlin',
    website: 'https://jane.example/me',
    aboutMe: 'Enthusiast of <b>HyperCS</b>.',
};

test('shows the public profile for a guest', async () => {
    const { mock } = page();
    mock.on('get', '/api/v1/users/{username}', 200, jane);
    expect(
        await screen.findByRole('heading', { name: 'Jane Doe' }),
    ).toBeInTheDocument();
    expect(
        mock.calls('get', '/api/v1/users/{username}')[0].params.username,
    ).toBe('jane');
    expect(screen.getByText('Moderator')).toBeInTheDocument();
    expect(screen.getByText('4 posts')).toBeInTheDocument();
    expect(screen.getByText('Berlin')).toBeInTheDocument();
    expect(screen.getByText(/joined/)).toBeInTheDocument();
    // About me is plain text, never HTML.
    expect(
        screen.getByText('Enthusiast of <b>HyperCS</b>.'),
    ).toBeInTheDocument();
    const site = screen.getByRole('link', { name: 'jane.example' });
    expect(site).toHaveAttribute('href', 'https://jane.example/me');
    expect(site).toHaveAttribute('rel', expect.stringContaining('noopener'));
    expect(site).toHaveAttribute('target', '_blank');
});

test('singular post count and no unsafe website link', async () => {
    const { mock } = page();
    mock.on('get', '/api/v1/users/{username}', 200, {
        ...jane,
        postCount: 1,
        website: 'javascript:alert(1)',
        aboutMe: null,
        location: null,
    });
    await screen.findByRole('heading', { name: 'Jane Doe' });
    expect(screen.getByText('1 post')).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: /alert/ })).toBeNull();
    expect(screen.queryByText('About me')).toBeNull();
});

test('unknown member shows a not-found state', async () => {
    const { mock } = page('/forum/users/ghost');
    mock.on('get', '/api/v1/users/{username}', 404, problem(404, 'not_found'));
    expect(await screen.findByRole('alert')).toHaveTextContent(
        'Member not found',
    );
    expect(
        screen.getByRole('link', { name: 'Back to the forum' }),
    ).toHaveAttribute('href', '/forum');
});

test('other failures offer a retry', async () => {
    const { user, mock } = page();
    mock.onUnlisted(
        'get',
        '/api/v1/users/{username}',
        500,
        problem(500, 'internal_error'),
    );
    expect(await screen.findByRole('alert')).toHaveTextContent(
        'Something went wrong',
    );
    mock.on('get', '/api/v1/users/{username}', 200, jane);
    await user.click(screen.getByRole('button', { name: 'Try again' }));
    expect(
        await screen.findByRole('heading', { name: 'Jane Doe' }),
    ).toBeInTheDocument();
});

test('German: pluralized post count', async () => {
    const { i18n } = await import('@/shared/i18n');
    await i18n.changeLanguage('de');
    const { mock } = page();
    mock.on('get', '/api/v1/users/{username}', 200, jane);
    expect(await screen.findByText('4 Beiträge')).toBeInTheDocument();
    expect(screen.getByText('Moderator')).toBeInTheDocument();
});
