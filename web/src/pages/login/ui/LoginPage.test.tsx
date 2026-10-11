import { screen } from '@testing-library/react';
import { Route, Routes } from 'react-router-dom';
import { expect, test } from 'vitest';
import { makeMe, mockApi, renderApp } from '../../../../test/utils';
import { LoginPage } from './LoginPage';

function routes() {
    return (
        <Routes>
            <Route path="/login" element={<LoginPage />} />
            <Route path="/settings" element={<p>settings screen</p>} />
            <Route path="/" element={<p>home screen</p>} />
        </Routes>
    );
}

test('guests see the login form with a link to registration', async () => {
    renderApp(routes(), { route: '/login' });
    expect(await screen.findByLabelText('Username')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Register' })).toHaveAttribute(
        'href',
        '/register',
    );
});

test('a logged-in user is sent to the page the guard came from', async () => {
    const { container } = renderApp(
        <Routes>
            <Route path="/login" element={<LoginPage />} />
            <Route path="/settings" element={<p>settings screen</p>} />
        </Routes>,
        {
            me: makeMe(),
            route: { pathname: '/login', state: { from: '/settings' } },
        },
    );
    expect(await screen.findByText('settings screen')).toBeInTheDocument();
    expect(container).not.toHaveTextContent('Username');
});

test('a logged-in user without a return target lands on the homepage', async () => {
    renderApp(routes(), { me: makeMe(), route: '/login', mock: mockApi() });
    expect(await screen.findByText('home screen')).toBeInTheDocument();
});
