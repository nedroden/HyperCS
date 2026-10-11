import { screen, waitFor } from '@testing-library/react';
import { expect, test, vi } from 'vitest';
import { i18n } from '@/shared/i18n';
import { makeMe, mockApi, problem, renderApp } from '../../../../test/utils';
import { SignUpForm } from './SignUpForm';

async function fillValid(user: ReturnType<typeof renderApp>['user']) {
    await user.type(screen.getByLabelText('Username'), 'alice');
    await user.type(screen.getByLabelText('Email address'), 'a@example.com');
    await user.type(screen.getByLabelText('Password'), 'correct horse battery');
    await user.type(
        screen.getByLabelText('Confirm password'),
        'correct horse battery',
    );
}

test('links the terms and enforces the client-side limits', async () => {
    const { user, mock } = renderApp(<SignUpForm />);
    expect(
        screen.getByRole('link', { name: 'terms of service' }),
    ).toHaveAttribute('href', '/terms');
    await user.type(screen.getByLabelText('Username'), 'a b');
    await user.type(screen.getByLabelText('Email address'), 'nope');
    await user.type(screen.getByLabelText('Password'), 'short');
    await user.type(screen.getByLabelText('Confirm password'), 'different');
    await user.click(screen.getByRole('button', { name: 'Register' }));
    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent('5 errors need your attention');
    expect(alert).toHaveTextContent('Usernames may only contain');
    expect(alert).toHaveTextContent('Enter a valid email address.');
    expect(alert).toHaveTextContent('The password is too short.');
    expect(alert).toHaveTextContent('The passwords do not match.');
    expect(alert).toHaveTextContent('You must accept the terms of service.');
    expect(mock.calls('post', '/api/v1/auth/register')).toHaveLength(0);
});

test('registers, logs in and stores the browser time zone and chosen language', async () => {
    vi.spyOn(Intl.DateTimeFormat.prototype, 'resolvedOptions').mockReturnValue({
        timeZone: 'Europe/Amsterdam',
    } as Intl.ResolvedDateTimeFormatOptions);
    await i18n.changeLanguage('de');
    const mock = mockApi();
    mock.on('post', '/api/v1/auth/register', 201, {
        id: '1',
        username: 'alice',
        displayName: 'alice',
        role: 'member',
    });
    mock.on('post', '/api/v1/auth/login', 204);
    mock.on('patch', '/api/v1/me/preferences', 204);
    const { user } = renderApp(<SignUpForm />, { mock });
    mock.on('get', '/api/v1/me', 200, makeMe({ language: 'de' }));
    await user.type(screen.getByLabelText('Benutzername'), 'alice');
    await user.type(screen.getByLabelText('E-Mail-Adresse'), 'a@example.com');
    await user.type(screen.getByLabelText('Passwort'), 'correct horse battery');
    await user.type(
        screen.getByLabelText('Passwort bestätigen'),
        'correct horse battery',
    );
    await user.click(screen.getByLabelText(/Ich stimme/));
    await user.click(screen.getByRole('button', { name: 'Registrieren' }));
    await waitFor(() =>
        expect(mock.calls('patch', '/api/v1/me/preferences')).toHaveLength(1),
    );
    expect(mock.calls('post', '/api/v1/auth/register')[0].body).toEqual({
        username: 'alice',
        email: 'a@example.com',
        password: 'correct horse battery',
        acceptTerms: true,
    });
    const patch = mock.calls('patch', '/api/v1/me/preferences')[0];
    expect(patch.body).toEqual({
        timezone: 'Europe/Amsterdam',
        language: 'de',
    });
    // The CSRF token from /me was installed before the PATCH.
    expect(patch.headers.get('x-csrf-token')).toBe('csrf-token-1');
    vi.restoreAllMocks();
});

test('shows server field errors (username_taken, email_taken)', async () => {
    const mock = mockApi();
    mock.on(
        'post',
        '/api/v1/auth/register',
        422,
        problem(422, 'validation_failed', [
            { field: 'username', code: 'username_taken' },
            { field: 'email', code: 'email_taken' },
        ]),
    );
    const { user } = renderApp(<SignUpForm />, { mock });
    await fillValid(user);
    await user.click(screen.getByLabelText(/I agree/));
    await user.click(screen.getByRole('button', { name: 'Register' }));
    await waitFor(() =>
        expect(screen.getByLabelText('Username')).toHaveAttribute(
            'aria-invalid',
            'true',
        ),
    );
    expect(
        await screen.findAllByText('This username is already taken.'),
    ).not.toHaveLength(0);
    expect(
        screen.getAllByText('This email address is already in use.').length,
    ).toBeGreaterThan(0);
    expect(mock.calls('post', '/api/v1/auth/login')).toHaveLength(0);
});
