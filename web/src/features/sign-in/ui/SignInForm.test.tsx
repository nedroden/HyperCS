import { screen, waitFor } from '@testing-library/react';
import { expect, test } from 'vitest';
import { makeMe, mockApi, problem, renderApp } from '../../../../test/utils';
import { SignInForm } from './SignInForm';

async function fill(
    user: ReturnType<typeof renderApp>['user'],
    username: string,
    password: string,
) {
    await user.type(screen.getByLabelText('Username'), username);
    await user.type(screen.getByLabelText('Password'), password);
}

test('validates required fields client side without calling the API', async () => {
    const { user, mock } = renderApp(<SignInForm />);
    await user.click(screen.getByRole('button', { name: 'Login' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
        '2 errors need your attention',
    );
    expect(screen.getByLabelText('Username')).toHaveAttribute(
        'aria-invalid',
        'true',
    );
    expect(mock.calls('post', '/api/v1/auth/login')).toHaveLength(0);
});

test('logs in with remember-me and reloads /me (rotated session, new CSRF token)', async () => {
    const mock = mockApi();
    mock.on('post', '/api/v1/auth/login', 204);
    const { user } = renderApp(<SignInForm />, { mock });
    await fill(user, 'alice', 'correct horse battery');
    await user.click(screen.getByLabelText('Remember me'));
    // After login /me returns the account.
    mock.on('get', '/api/v1/me', 200, makeMe({ csrfToken: 'after-login' }));
    await user.click(screen.getByRole('button', { name: 'Login' }));
    await waitFor(() =>
        expect(mock.calls('post', '/api/v1/auth/login')).toHaveLength(1),
    );
    expect(mock.calls('post', '/api/v1/auth/login')[0].body).toEqual({
        username: 'alice',
        password: 'correct horse battery',
        remember: true,
    });
    await waitFor(() =>
        expect(mock.calls('get', '/api/v1/me').length).toBeGreaterThan(1),
    );
});

test('shows invalid_credentials from the API as a form error and keeps the form usable', async () => {
    const mock = mockApi();
    mock.on(
        'post',
        '/api/v1/auth/login',
        401,
        problem(401, 'invalid_credentials'),
    );
    const { user } = renderApp(<SignInForm />, { mock });
    await fill(user, 'alice', 'wrong password');
    await user.click(screen.getByRole('button', { name: 'Login' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
        'Username or password is incorrect.',
    );
    expect(screen.getByRole('button', { name: 'Login' })).toBeEnabled();
});
