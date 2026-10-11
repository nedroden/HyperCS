import { screen, waitFor, within } from '@testing-library/react';
import { expect, test } from 'vitest';
import { makeMe, mockApi, problem, renderApp } from '../../../../test/utils';
import { Header } from './Header';

function nav() {
    return screen.getByRole('navigation', { name: 'Main navigation' });
}

test('guest: Home, Forum, Login in the menu; Login | Register in the user box', async () => {
    renderApp(<Header />);
    expect(
        await within(nav()).findByRole('link', { name: 'Login' }),
    ).toHaveAttribute('href', '/login');
    expect(
        within(nav())
            .getAllByRole('link')
            .map((a) => a.textContent),
    ).toEqual(['Home', 'Forum', 'Login']);
    expect(within(nav()).queryByRole('button')).toBeNull();
    expect(screen.getByRole('link', { name: 'Register' })).toHaveAttribute(
        'href',
        '/register',
    );
});

test('member: Home, Forum, Logout in the menu; Hello, <display name> | Logout in the user box', async () => {
    renderApp(<Header />, { me: makeMe({ displayName: 'Alice A.' }) });
    expect(
        await screen.findByRole('link', { name: 'Alice A.' }),
    ).toHaveAttribute('href', '/settings');
    expect(
        within(nav())
            .getAllByRole('link')
            .map((a) => a.textContent),
    ).toEqual(['Home', 'Forum']);
    expect(
        within(nav()).getByRole('button', { name: 'Logout' }),
    ).toBeInTheDocument();
    expect(screen.getAllByRole('button', { name: 'Logout' })).toHaveLength(2);
    expect(screen.queryByRole('link', { name: 'Login' })).toBeNull();
    expect(screen.queryByRole('link', { name: 'Register' })).toBeNull();
});

test('shows neither guest nor member links while the session loads', () => {
    const mock = mockApi();
    mock.on('get', '/api/v1/me', 401, problem(401, 'unauthorized'));
    renderApp(<Header />, { mock });
    expect(screen.queryByRole('link', { name: 'Login' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Logout' })).toBeNull();
});

test('logging out calls the API with the CSRF token and turns the header into the guest header', async () => {
    const mock = mockApi();
    mock.on('post', '/api/v1/auth/logout', 204);
    const { user } = renderApp(<Header />, { me: makeMe(), mock });
    await screen.findByRole('link', { name: 'Alice' });
    await user.click(within(nav()).getByRole('button', { name: 'Logout' }));
    await waitFor(() =>
        expect(
            within(nav()).getByRole('link', { name: 'Login' }),
        ).toBeInTheDocument(),
    );
    const [call] = mock.calls('post', '/api/v1/auth/logout');
    expect(call.headers.get('x-csrf-token')).toBe('csrf-token-1');
});
