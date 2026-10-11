import { screen, waitFor, within } from '@testing-library/react';
import { expect, test } from 'vitest';
import { makeMe, problem, renderApp } from '../../../../test/utils';
import { describeDevice } from '../model/describeDevice';
import { SessionsPanel } from './SessionsPanel';

const CHROME_MAC =
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36';
const SAFARI_IPHONE =
    'Mozilla/5.0 (iPhone; CPU iPhone OS 17_5 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Mobile/15E148 Safari/604.1';

const ago = (hours: number) =>
    new Date(Date.now() - hours * 3_600_000).toISOString();

const sessions = [
    {
        id: '0190c3a2-0000-7000-8000-0000000000a1',
        createdAt: ago(24 * 7),
        lastSeenAt: ago(0.01),
        expiresAt: ago(-24 * 30),
        remember: true,
        current: true,
        userAgent: CHROME_MAC,
    },
    {
        id: '0190c3a2-0000-7000-8000-0000000000a2',
        createdAt: ago(48),
        lastSeenAt: ago(1),
        expiresAt: ago(-24),
        remember: false,
        current: false,
        userAgent: SAFARI_IPHONE,
    },
];

test('describeDevice recognizes common browsers and systems', () => {
    expect(describeDevice(CHROME_MAC)).toEqual({
        browser: 'Chrome',
        os: 'macOS',
    });
    expect(describeDevice(SAFARI_IPHONE)).toEqual({
        browser: 'Safari',
        os: 'iPhone',
    });
    expect(describeDevice(null)).toEqual({ browser: null, os: null });
});

test('lists sessions, marks this device and offers revoke for the others only', async () => {
    const { mock } = renderApp(<SessionsPanel />, { me: makeMe() });
    mock.on('get', '/api/v1/me/sessions', 200, sessions);
    const rows = await screen.findAllByRole('row');
    expect(rows).toHaveLength(3);
    expect(rows[1]).toHaveTextContent('Chrome on macOS');
    expect(rows[1]).toHaveTextContent('This device');
    expect(within(rows[1]).queryByRole('button')).toBeNull();
    expect(rows[2]).toHaveTextContent('Safari on iPhone');
    expect(within(rows[2]).getByRole('button')).toHaveAccessibleName(
        'Revoke session: Safari on iPhone',
    );
    // Created and last-seen are relative times with the absolute time as tooltip.
    expect(rows[2].querySelectorAll('time[title]')).toHaveLength(2);
});

test('revoking a session calls the API and refreshes the list', async () => {
    const { user, mock } = renderApp(<SessionsPanel />, { me: makeMe() });
    mock.on('get', '/api/v1/me/sessions', 200, sessions);
    mock.on('delete', '/api/v1/me/sessions/{id}', 204);
    const revoke = await screen.findByRole('button', {
        name: 'Revoke session: Safari on iPhone',
    });
    mock.on('get', '/api/v1/me/sessions', 200, [sessions[0]]);
    await user.click(revoke);
    await waitFor(() =>
        expect(mock.calls('delete', '/api/v1/me/sessions/{id}')).toHaveLength(
            1,
        ),
    );
    expect(mock.calls('delete', '/api/v1/me/sessions/{id}')[0].params.id).toBe(
        sessions[1].id,
    );
    await waitFor(() => expect(screen.getAllByRole('row')).toHaveLength(2));
    expect(screen.getByRole('status')).toHaveTextContent(
        'The session was revoked.',
    );
});

test('a failed revoke shows the translated error', async () => {
    const { user, mock } = renderApp(<SessionsPanel />, { me: makeMe() });
    mock.on('get', '/api/v1/me/sessions', 200, sessions);
    mock.on(
        'delete',
        '/api/v1/me/sessions/{id}',
        404,
        problem(404, 'not_found'),
    );
    await user.click(
        await screen.findByRole('button', { name: /Revoke session/ }),
    );
    expect(await screen.findByRole('alert')).toHaveTextContent(
        'The requested item could not be found.',
    );
});

test('shows an error state with retry when the list fails', async () => {
    const { user, mock } = renderApp(<SessionsPanel />, { me: makeMe() });
    mock.onUnlisted(
        'get',
        '/api/v1/me/sessions',
        500,
        problem(500, 'internal_error'),
    );
    expect(await screen.findByRole('alert')).toHaveTextContent(
        'Something went wrong',
    );
    mock.on('get', '/api/v1/me/sessions', 200, sessions);
    await user.click(screen.getByRole('button', { name: 'Try again' }));
    expect(await screen.findByText('Chrome on macOS')).toBeInTheDocument();
});
