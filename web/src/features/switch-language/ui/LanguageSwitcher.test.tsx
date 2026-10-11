import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, test } from 'vitest';
import { i18n } from '@/shared/i18n';
import { makeMe, mockApi, renderApp } from '../../../../test/utils';
import { LanguageSwitcher } from './LanguageSwitcher';

function setup() {
    renderApp(
        <>
            <button>outside</button>
            <LanguageSwitcher />
        </>,
    );
    return screen.getByRole('button', { name: /Language: English/ });
}

test('switches the language to German and returns focus to the toggle', async () => {
    const toggle = setup();
    await userEvent.click(toggle);
    await userEvent.click(
        screen.getByRole('menuitemradio', { name: 'Deutsch' }),
    );
    expect(i18n.language).toBe('de');
    expect(document.documentElement.lang).toBe('de');
    expect(screen.queryByRole('menu')).toBeNull();
    expect(
        screen.getByRole('button', { name: /Sprache: Deutsch/ }),
    ).toHaveFocus();
});

test('exposes aria-expanded and focuses the current language when opened', async () => {
    const toggle = setup();
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
    await userEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-expanded', 'true');
    expect(toggle).toHaveAttribute('aria-haspopup', 'menu');
    expect(
        screen.getByRole('menuitemradio', { name: 'English (US)' }),
    ).toHaveFocus();
    expect(
        screen.getByRole('menuitemradio', { name: 'English (US)' }),
    ).toHaveAttribute('aria-checked', 'true');
});

test('Escape closes the menu and returns focus to the toggle', async () => {
    const toggle = setup();
    await userEvent.click(toggle);
    await userEvent.keyboard('{Escape}');
    expect(screen.queryByRole('menu')).toBeNull();
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
    expect(toggle).toHaveFocus();
});

test('arrow keys, Home and End move between languages, wrapping around', async () => {
    const toggle = setup();
    toggle.focus();
    await userEvent.keyboard('{ArrowDown}');
    const en = screen.getByRole('menuitemradio', { name: 'English (US)' });
    const de = screen.getByRole('menuitemradio', { name: 'Deutsch' });
    expect(en).toHaveFocus();
    await userEvent.keyboard('{ArrowDown}');
    expect(de).toHaveFocus();
    await userEvent.keyboard('{ArrowDown}');
    expect(en).toHaveFocus();
    await userEvent.keyboard('{ArrowUp}');
    expect(de).toHaveFocus();
    await userEvent.keyboard('{Home}');
    expect(en).toHaveFocus();
    await userEvent.keyboard('{End}');
    expect(de).toHaveFocus();
});

test('Enter on a focused item selects it', async () => {
    const toggle = setup();
    toggle.focus();
    await userEvent.keyboard('{ArrowDown}{ArrowDown}{Enter}');
    expect(i18n.language).toBe('de');
});

test('clicking outside closes the menu', async () => {
    const toggle = setup();
    await userEvent.click(toggle);
    await userEvent.click(screen.getByRole('button', { name: 'outside' }));
    expect(screen.queryByRole('menu')).toBeNull();
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
});

test('Tab closes the menu', async () => {
    const toggle = setup();
    await userEvent.click(toggle);
    await userEvent.tab();
    expect(screen.queryByRole('menu')).toBeNull();
});

test('guests: the choice is remembered in localStorage and nothing is sent to the API', async () => {
    const mock = mockApi();
    const { user } = renderApp(<LanguageSwitcher />, { mock });
    await user.click(screen.getByRole('button', { name: /Language/ }));
    await user.click(screen.getByRole('menuitemradio', { name: 'Deutsch' }));
    expect(localStorage.getItem('hypercs.language')).toBe('de');
    expect(mock.calls('patch', '/api/v1/me/preferences')).toHaveLength(0);
});

test('logged-in users: the choice is also saved to their profile', async () => {
    const mock = mockApi();
    mock.on('patch', '/api/v1/me/preferences', 204);
    const { user } = renderApp(<LanguageSwitcher />, {
        mock,
        me: makeMe({ language: 'en-US' }),
    });
    // Wait until the session is known, then switch.
    await waitFor(() =>
        expect(mock.calls('get', '/api/v1/me').length).toBeGreaterThan(0),
    );
    await user.click(screen.getByRole('button', { name: /Language/ }));
    await user.click(screen.getByRole('menuitemradio', { name: 'Deutsch' }));
    await waitFor(() =>
        expect(mock.calls('patch', '/api/v1/me/preferences')).toHaveLength(1),
    );
    const [call] = mock.calls('patch', '/api/v1/me/preferences');
    expect(call.body).toEqual({ language: 'de' });
    expect(call.headers.get('x-csrf-token')).toBe('csrf-token-1');
    expect(localStorage.getItem('hypercs.language')).toBe('de');
});

test('a stored profile language is applied once when the user logs in', async () => {
    renderApp(<LanguageSwitcher />, { me: makeMe({ language: 'de' }) });
    expect(
        await screen.findByRole('button', { name: /Sprache: Deutsch/ }),
    ).toBeInTheDocument();
    expect(i18n.language).toBe('de');
});
