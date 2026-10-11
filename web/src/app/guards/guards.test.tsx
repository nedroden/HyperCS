import { screen } from '@testing-library/react';
import { Route, Routes, useLocation } from 'react-router-dom';
import { expect, test } from 'vitest';
import { makeMe, mockApi, renderApp } from '../../../test/utils';
import { RequireAuth } from './RequireAuth';
import { RequireRole } from './RequireRole';

function LoginProbe() {
    const { state } = useLocation();
    return <p>login screen from {JSON.stringify(state)}</p>;
}

function routes() {
    return (
        <Routes>
            <Route path="/login" element={<LoginProbe />} />
            <Route element={<RequireAuth />}>
                <Route path="/settings" element={<p>settings screen</p>} />
            </Route>
            <Route element={<RequireRole role="admin" />}>
                <Route path="/admin" element={<p>admin screen</p>} />
            </Route>
        </Routes>
    );
}

test('RequireAuth redirects guests to /login and remembers where they wanted to go', async () => {
    renderApp(routes(), { route: '/settings?tab=1' });
    expect(await screen.findByText(/login screen from/)).toHaveTextContent(
        '{"from":"/settings?tab=1"}',
    );
    expect(screen.queryByText('settings screen')).toBeNull();
});

test('RequireAuth shows content to logged-in users and nothing while loading', async () => {
    renderApp(routes(), { route: '/settings', me: makeMe() });
    expect(screen.queryByText('settings screen')).toBeNull();
    expect(await screen.findByText('settings screen')).toBeInTheDocument();
});

test('RequireAuth shows a retryable error when the session check fails', async () => {
    const mock = mockApi();
    const { user } = renderApp(routes(), {
        route: '/settings',
        me: 'error',
        mock,
    });
    expect(
        await screen.findByRole('button', { name: /try again|retry/i }),
    ).toBeInTheDocument();
    mock.on('get', '/api/v1/me', 200, makeMe());
    await user.click(screen.getByRole('button', { name: /try again|retry/i }));
    expect(await screen.findByText('settings screen')).toBeInTheDocument();
});

test('RequireRole: members get an access-denied state, admins get the page', async () => {
    const member = renderApp(routes(), { route: '/admin', me: makeMe() });
    expect(await screen.findByText('Access denied')).toBeInTheDocument();
    expect(screen.queryByText('admin screen')).toBeNull();
    member.unmount();

    renderApp(routes(), { route: '/admin', me: makeMe({ role: 'admin' }) });
    expect(await screen.findByText('admin screen')).toBeInTheDocument();
});

test('RequireRole sends guests to the login page', async () => {
    renderApp(routes(), { route: '/admin' });
    expect(await screen.findByText(/login screen from/)).toBeInTheDocument();
});
