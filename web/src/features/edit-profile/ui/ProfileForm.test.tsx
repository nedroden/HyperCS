import { screen, waitFor } from '@testing-library/react';
import { expect, test } from 'vitest';
import { makeMe, problem, renderApp } from '../../../../test/utils';
import { AccountForm } from './AccountForm';
import { ProfileForm } from './ProfileForm';

const me = makeMe({
    displayName: 'Alice',
    location: 'Amsterdam',
    website: null,
    aboutMe: 'Hi there',
    firstName: 'Alice',
    lastName: null,
});

test('profile: prefilled, only changed fields are sent, success is announced', async () => {
    const { user, mock } = renderApp(<ProfileForm />, { me });
    mock.on('patch', '/api/v1/me/profile', 204);
    const name = await screen.findByLabelText('Display name');
    expect(name).toHaveValue('Alice');
    expect(screen.getByLabelText('Location')).toHaveValue('Amsterdam');
    expect(screen.getByLabelText('About me')).toHaveValue('Hi there');
    // Avatar upload/remove controls are hidden in the MVP.
    expect(screen.queryByText(/Upload/)).toBeNull();

    await user.clear(name);
    await user.type(name, 'Alice B.');
    await user.type(screen.getByLabelText('Website'), 'https://alice.dev');
    await user.click(screen.getByRole('button', { name: 'Save profile' }));

    expect(await screen.findByRole('status')).toHaveTextContent('Saved.');
    expect(mock.calls('patch', '/api/v1/me/profile')[0].body).toEqual({
        displayName: 'Alice B.',
        website: 'https://alice.dev',
    });
});

test('profile: clearing an optional field sends null', async () => {
    const { user, mock } = renderApp(<ProfileForm />, { me });
    mock.on('patch', '/api/v1/me/profile', 204);
    await user.clear(await screen.findByLabelText('Location'));
    await user.click(screen.getByRole('button', { name: 'Save profile' }));
    await screen.findByText('Saved.');
    expect(mock.calls('patch', '/api/v1/me/profile')[0].body).toEqual({
        location: null,
    });
});

test('profile: refreshes /me after saving', async () => {
    const { user, mock } = renderApp(<ProfileForm />, { me });
    mock.on('patch', '/api/v1/me/profile', 204);
    await user.type(await screen.findByLabelText('Location'), '!');
    await user.click(screen.getByRole('button', { name: 'Save profile' }));
    await screen.findByText('Saved.');
    expect(mock.calls('get', '/api/v1/me')).toHaveLength(2);
});

test('profile: validates limits and URL scheme without calling the API', async () => {
    const { user, mock } = renderApp(<ProfileForm />, { me });
    const name = await screen.findByLabelText('Display name');
    await user.clear(name);
    await user.type(screen.getByLabelText('Website'), 'ftp://example.com');
    await user.click(screen.getByRole('button', { name: 'Save profile' }));
    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent('This field is required.');
    expect(alert).toHaveTextContent('Enter a valid URL.');
    expect(name).toHaveAttribute('aria-invalid', 'true');
    expect(mock.calls('patch', '/api/v1/me/profile')).toHaveLength(0);
});

test('profile: without changes nothing is sent', async () => {
    const { user, mock } = renderApp(<ProfileForm />, { me });
    await screen.findByLabelText('Display name');
    await user.click(screen.getByRole('button', { name: 'Save profile' }));
    expect(await screen.findByRole('status')).toHaveTextContent(
        'There is nothing to save.',
    );
    expect(mock.calls('patch', '/api/v1/me/profile')).toHaveLength(0);
});

test('profile: maps server field errors onto the fields', async () => {
    const { user, mock } = renderApp(<ProfileForm />, { me });
    mock.on(
        'patch',
        '/api/v1/me/profile',
        422,
        problem(422, 'validation_failed', [{ field: 'website', code: 'url' }]),
    );
    await user.type(await screen.findByLabelText('Website'), 'https://x.io');
    await user.click(screen.getByRole('button', { name: 'Save profile' }));
    await waitFor(() =>
        expect(screen.getByLabelText('Website')).toHaveAttribute(
            'aria-invalid',
            'true',
        ),
    );
    expect(document.getElementById('website-error')).toHaveTextContent(
        'Enter a valid URL.',
    );
    expect(screen.queryByText('Saved.')).toBeNull();
});

test('account: username is read-only, email change sends only the email', async () => {
    const { user, mock } = renderApp(<AccountForm />, { me });
    mock.on('patch', '/api/v1/me/account', 204);
    const email = await screen.findByLabelText('Email address');
    expect(screen.getByLabelText('Username')).toBeDisabled();
    expect(screen.getByLabelText('Username')).toHaveValue('alice');
    expect(screen.getByLabelText('First name')).toHaveValue('Alice');
    await user.clear(email);
    await user.type(email, 'new@example.com');
    await user.click(screen.getByRole('button', { name: 'Save account' }));
    await screen.findByText('Saved.');
    expect(mock.calls('patch', '/api/v1/me/account')[0].body).toEqual({
        email: 'new@example.com',
    });
});

test('account: invalid email is rejected client-side, taken email server-side', async () => {
    const { user, mock } = renderApp(<AccountForm />, { me });
    const email = await screen.findByLabelText('Email address');
    await user.clear(email);
    await user.type(email, 'nope');
    await user.click(screen.getByRole('button', { name: 'Save account' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
        'Enter a valid email address.',
    );
    expect(mock.calls('patch', '/api/v1/me/account')).toHaveLength(0);

    mock.on(
        'patch',
        '/api/v1/me/account',
        422,
        problem(422, 'validation_failed', [
            { field: 'email', code: 'email_taken' },
        ]),
    );
    await user.clear(email);
    await user.type(email, 'taken@example.com');
    await user.click(screen.getByRole('button', { name: 'Save account' }));
    await waitFor(() =>
        expect(document.getElementById('email-error')).toHaveTextContent(
            'This email address is already in use.',
        ),
    );
});

test('account: clearing the first name sends null', async () => {
    const { user, mock } = renderApp(<AccountForm />, { me });
    mock.on('patch', '/api/v1/me/account', 204);
    await user.clear(await screen.findByLabelText('First name'));
    await user.click(screen.getByRole('button', { name: 'Save account' }));
    await screen.findByText('Saved.');
    expect(mock.calls('patch', '/api/v1/me/account')[0].body).toEqual({
        firstName: null,
    });
});
