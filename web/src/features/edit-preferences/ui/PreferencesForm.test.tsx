import { screen, waitFor } from '@testing-library/react';
import { afterEach, expect, test, vi } from 'vitest';
import { i18n } from '@/shared/i18n';
import { makeMe, problem, renderApp } from '../../../../test/utils';
import { timeZoneOptions } from '../model/timeZones';
import { PreferencesForm } from './PreferencesForm';

afterEach(() => vi.restoreAllMocks());

const me = makeMe({
    timezone: 'Europe/Amsterdam',
    notifyOnReply: true,
    notifyOnQuote: false,
    showOnline: true,
});

test('time zone list: full IANA list, UTC and the stored zone included, with a fallback', () => {
    const zones = timeZoneOptions('Mars/Olympus');
    expect(zones).toContain('Europe/Amsterdam');
    expect(zones).toContain('UTC');
    expect(zones).toContain('Mars/Olympus');
    expect(zones.length).toBeGreaterThan(100);
    vi.spyOn(Intl, 'supportedValuesOf').mockImplementation(() => {
        throw new Error('unsupported');
    });
    const fallback = timeZoneOptions();
    expect(fallback).toContain('Europe/Berlin');
    expect(fallback).toContain('UTC');
});

test('shows the stored preferences and sends only the changed ones', async () => {
    const { user, mock } = renderApp(<PreferencesForm />, { me });
    mock.on('patch', '/api/v1/me/preferences', 204);
    const zone = await screen.findByLabelText('Time zone');
    expect(zone).toHaveValue('Europe/Amsterdam');
    expect(
        screen.getByLabelText('Email me when someone replies to my topics'),
    ).toBeChecked();
    expect(
        screen.getByLabelText('Email me when someone quotes my posts'),
    ).not.toBeChecked();

    await user.selectOptions(zone, 'America/New_York');
    await user.click(
        screen.getByLabelText('Email me when someone quotes my posts'),
    );
    await user.click(
        screen.getByLabelText('Show me in the list of users online'),
    );
    await user.click(screen.getByRole('button', { name: 'Save preferences' }));

    expect(await screen.findByRole('status')).toHaveTextContent('Saved.');
    expect(mock.calls('patch', '/api/v1/me/preferences')[0].body).toEqual({
        timezone: 'America/New_York',
        notifyOnQuote: true,
        showOnline: false,
    });
});

test('maps a server error for the time zone onto the field', async () => {
    const { user, mock } = renderApp(<PreferencesForm />, { me });
    mock.on(
        'patch',
        '/api/v1/me/preferences',
        422,
        problem(422, 'validation_failed', [
            { field: 'timezone', code: 'invalid_timezone' },
        ]),
    );
    await user.selectOptions(await screen.findByLabelText('Time zone'), 'UTC');
    await user.click(screen.getByRole('button', { name: 'Save preferences' }));
    await waitFor(() =>
        expect(document.getElementById('timezone-error')).toHaveTextContent(
            'This time zone is not valid.',
        ),
    );
});

test('the language dropdown applies immediately, is not part of the save body and is stored', async () => {
    const { user, mock } = renderApp(<PreferencesForm />, { me });
    mock.on('patch', '/api/v1/me/preferences', 204);
    const language = await screen.findByLabelText('Language');
    expect(language).toHaveValue('en-US');
    await user.selectOptions(language, 'de');
    expect(i18n.language).toBe('de');
    await waitFor(() =>
        expect(mock.calls('patch', '/api/v1/me/preferences')).toHaveLength(1),
    );
    expect(mock.calls('patch', '/api/v1/me/preferences')[0].body).toEqual({
        language: 'de',
    });
    expect(await screen.findByLabelText('Sprache')).toHaveValue('de');
});
