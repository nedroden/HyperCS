import { screen, waitFor, within } from '@testing-library/react';
import { expect, test } from 'vitest';
import { Footer } from '@/widgets/footer';
import { i18n } from '@/shared/i18n';
import { makeMe, renderApp } from '../../../../test/utils';
import { SettingsPage } from './SettingsPage';

async function openPreferences(user: ReturnType<typeof setup>['user']) {
    await screen.findByLabelText('Display name');
    await user.click(screen.getByRole('link', { name: 'Preferences' }));
}

function setup(language = 'en-US') {
    window.location.hash = ''; // links to a section change it, and jsdom keeps it between tests
    const result = renderApp(
        <>
            <SettingsPage />
            <Footer />
        </>,
        { me: makeMe({ language }) },
    );
    result.mock.on('patch', '/api/v1/me/preferences', 204);
    result.mock.on('get', '/api/v1/me/sessions', 200, []);
    return result;
}

test('renders every section with a sidebar linking to it', async () => {
    setup();
    await screen.findByLabelText('Display name');
    const nav = screen.getByRole('navigation', { name: 'Settings sections' });
    for (const name of [
        'Profile',
        'Account',
        'Password',
        'Preferences',
        'Active sessions',
    ]) {
        expect(within(nav).getByRole('link', { name })).toBeInTheDocument();
    }
    expect(within(nav).getByRole('link', { name: 'Profile' })).toHaveClass(
        'active',
    );
});

test('shows only the selected section', async () => {
    const { user } = setup();
    await screen.findByLabelText('Display name');
    expect(screen.getAllByRole('heading', { level: 3 })).toHaveLength(1);
    expect(screen.queryByLabelText('Language')).toBeNull();

    const nav = screen.getByRole('navigation', { name: 'Settings sections' });
    await user.click(within(nav).getByRole('link', { name: 'Preferences' }));
    expect(await screen.findByLabelText('Language')).toBeInTheDocument();
    expect(screen.queryByLabelText('Display name')).toBeNull();
    expect(screen.getAllByRole('heading', { level: 3 })).toHaveLength(1);
    expect(within(nav).getByRole('link', { name: 'Preferences' })).toHaveClass(
        'active',
    );
});

test('the footer switcher updates the Preferences dropdown', async () => {
    const { user, mock } = setup();
    await openPreferences(user);
    expect(await screen.findByLabelText('Language')).toHaveValue('en-US');
    await user.click(screen.getByRole('button', { name: /Language: English/ }));
    await user.click(screen.getByRole('menuitemradio', { name: 'Deutsch' }));
    expect(await screen.findByLabelText('Sprache')).toHaveValue('de');
    await waitFor(() =>
        expect(mock.calls('patch', '/api/v1/me/preferences')).toHaveLength(1),
    );
    expect(mock.calls('patch', '/api/v1/me/preferences')[0].body).toEqual({
        language: 'de',
    });
});

test('the Preferences dropdown updates the footer switcher', async () => {
    const { user } = setup();
    await openPreferences(user);
    await user.selectOptions(await screen.findByLabelText('Language'), 'de');
    expect(
        await screen.findByRole('button', { name: /Sprache: Deutsch/ }),
    ).toBeInTheDocument();
    expect(i18n.language).toBe('de');
});
