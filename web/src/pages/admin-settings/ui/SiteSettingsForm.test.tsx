import { screen, waitFor } from '@testing-library/react';
import { expect, test } from 'vitest';
import { makeMe, mockApi, problem, renderApp } from '../../../../test/utils';
import { SiteSettingsForm } from './SiteSettingsForm';

const SETTINGS = {
    siteTitle: 'My Club',
    slogan: 'Come on in',
    defaultLanguage: 'en-US',
};

function setup() {
    const mock = mockApi();
    mock.on('get', '/api/v1/cms/site', 200, SETTINGS);
    mock.on('put', '/api/v1/cms/site', 204);
    return {
        ...renderApp(<SiteSettingsForm />, {
            mock,
            me: makeMe({ role: 'admin' }),
        }),
    };
}

test('shows the current settings', async () => {
    setup();
    expect(await screen.findByLabelText('Site title')).toHaveValue('My Club');
    expect(screen.getByLabelText('Slogan')).toHaveValue('Come on in');
    expect(screen.getByLabelText('Default language')).toHaveValue('en-US');
});

test('saves the changed settings', async () => {
    const { mock, user } = setup();
    const title = await screen.findByLabelText('Site title');
    await user.clear(title);
    await user.type(title, 'Our Club');
    await user.selectOptions(screen.getByLabelText('Default language'), 'de');
    await user.click(screen.getByRole('button', { name: 'Save settings' }));

    expect(await screen.findByText('Saved.')).toBeInTheDocument();
    expect(mock.calls('put', '/api/v1/cms/site')[0].body).toEqual({
        siteTitle: 'Our Club',
        slogan: 'Come on in',
        defaultLanguage: 'de',
    });
});

test('rejects an empty title without calling the API', async () => {
    const { mock, user } = setup();
    await user.clear(await screen.findByLabelText('Site title'));
    await user.click(screen.getByRole('button', { name: 'Save settings' }));

    expect(
        await screen.findAllByText('This field is required.'),
    ).not.toHaveLength(0);
    expect(mock.calls('put', '/api/v1/cms/site')).toHaveLength(0);
});

test('shows a field error reported by the API', async () => {
    const { mock, user } = setup();
    mock.onUnlisted(
        'put',
        '/api/v1/cms/site',
        422,
        problem(422, 'validation_failed', [
            { field: 'slogan', code: 'invalid_characters' },
        ]),
    );
    await screen.findByLabelText('Slogan');
    await user.click(screen.getByRole('button', { name: 'Save settings' }));
    await waitFor(() =>
        expect(screen.getByLabelText('Slogan')).toHaveAttribute(
            'aria-invalid',
            'true',
        ),
    );
});
