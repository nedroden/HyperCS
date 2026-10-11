import { waitFor } from '@testing-library/react';
import { expect, test } from 'vitest';
import { i18n } from '@/shared/i18n';
import { makeMe, mockApi, renderApp } from '../../../test/utils';
import { useSiteSettingsEffects } from './useSiteSettingsEffects';

function Probe() {
    useSiteSettingsEffects();
    return null;
}

function settings(defaultLanguage: string) {
    const mock = mockApi();
    mock.on('get', '/api/v1/cms/site', 200, {
        siteTitle: 'My Club',
        slogan: 's',
        defaultLanguage,
    });
    return mock;
}

test('sets the tab title to the site title', async () => {
    renderApp(<Probe />, { mock: settings('en-US') });
    await waitFor(() => expect(document.title).toBe('My Club'));
});

test('a guest who has not chosen a language gets the default, without it being remembered', async () => {
    renderApp(<Probe />, { mock: settings('de') });
    await waitFor(() => expect(i18n.language).toBe('de'));
    expect(localStorage.getItem('hypercs.language')).toBeNull();
});

test('a guest who has chosen a language keeps it', async () => {
    localStorage.setItem('hypercs.language', 'en-US');
    renderApp(<Probe />, { mock: settings('de') });
    await waitFor(() => expect(document.title).toBe('My Club'));
    expect(i18n.language).toBe('en-US');
});

test('a logged-in user is not switched to the default', async () => {
    renderApp(<Probe />, {
        mock: settings('de'),
        me: makeMe({ language: 'en-US' }),
    });
    await waitFor(() => expect(document.title).toBe('My Club'));
    expect(i18n.language).toBe('en-US');
});
