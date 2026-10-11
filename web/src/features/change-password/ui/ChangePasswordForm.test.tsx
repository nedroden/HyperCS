import { screen, waitFor } from '@testing-library/react';
import { expect, test } from 'vitest';
import { makeMe, problem, renderApp } from '../../../../test/utils';
import { ChangePasswordForm } from './ChangePasswordForm';

async function fill(
    user: ReturnType<typeof renderApp>['user'],
    current: string,
    next: string,
    confirm = next,
) {
    await user.type(await screen.findByLabelText('Current password'), current);
    await user.type(screen.getByLabelText('New password'), next);
    await user.type(screen.getByLabelText('Confirm new password'), confirm);
    await user.click(screen.getByRole('button', { name: 'Change password' }));
}

test('explains that other sessions are signed out', () => {
    renderApp(<ChangePasswordForm />, { me: makeMe() });
    expect(
        screen.getByText(/signs you out on all your other devices/),
    ).toBeInTheDocument();
});

test('validates length and confirmation (no composition rules)', async () => {
    const { user, mock } = renderApp(<ChangePasswordForm />, { me: makeMe() });
    await fill(user, 'old-password', 'short', 'different');
    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent('The password is too short.');
    expect(alert).toHaveTextContent('The passwords do not match.');
    expect(mock.calls('patch', '/api/v1/me/password')).toHaveLength(0);
});

test('accepts an all-lowercase 10 character password, confirms and clears the form', async () => {
    const { user, mock } = renderApp(<ChangePasswordForm />, { me: makeMe() });
    mock.on('patch', '/api/v1/me/password', 204);
    mock.on('get', '/api/v1/me/sessions', 200, []);
    await fill(user, 'old-password', 'abcdefghij');
    expect(await screen.findByRole('status')).toHaveTextContent(
        'Your password was changed. All other devices were signed out.',
    );
    expect(mock.calls('patch', '/api/v1/me/password')[0].body).toEqual({
        currentPassword: 'old-password',
        newPassword: 'abcdefghij',
    });
    expect(screen.getByLabelText('Current password')).toHaveValue('');
    expect(screen.getByLabelText('New password')).toHaveValue('');
});

test('a wrong current password is shown on that field and nothing is confirmed', async () => {
    const { user, mock } = renderApp(<ChangePasswordForm />, { me: makeMe() });
    mock.on(
        'patch',
        '/api/v1/me/password',
        422,
        problem(422, 'validation_failed', [
            { field: 'currentPassword', code: 'invalid_password' },
        ]),
    );
    await fill(user, 'wrong-password', 'abcdefghij');
    await waitFor(() =>
        expect(
            document.getElementById('currentPassword-error'),
        ).toHaveTextContent('The password is incorrect.'),
    );
    expect(screen.getByLabelText('Current password')).toHaveAttribute(
        'aria-invalid',
        'true',
    );
    expect(screen.getByRole('status')).toBeEmptyDOMElement();
});
