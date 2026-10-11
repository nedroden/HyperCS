import { screen, waitFor, within } from '@testing-library/react';
import { expect, test } from 'vitest';
import { mockApi, problem, renderApp } from '../../../../test/utils';
import { HomepageEditor } from './HomepageEditor';

const HOMEPAGE = {
    heroHeading: 'Welcome',
    heroButtonLabel: 'Join',
    features: [
        { title: 'Fast', text: 'Quick.' },
        { title: 'Open', text: 'Free.' },
        { title: 'Friendly', text: 'Nice.' },
    ],
};

function setup() {
    const mock = mockApi();
    mock.on('get', '/api/v1/cms/homepage', 200, HOMEPAGE);
    mock.on('get', '/api/v1/cms/terms', 200, { body: 'Be nice.' });
    mock.on('put', '/api/v1/cms/homepage', 204);
    mock.on('put', '/api/v1/cms/terms', 204);
    return renderApp(<HomepageEditor />, { mock });
}

test('saving the hero PUTs the whole document, keeping the features', async () => {
    const { mock, user } = setup();
    const heading = await screen.findByLabelText('Heading');
    expect(heading).toHaveValue('Welcome');
    await user.clear(heading);
    await user.type(heading, 'Hello world');
    await user.click(screen.getByRole('button', { name: 'Save hero' }));
    await waitFor(() =>
        expect(mock.calls('put', '/api/v1/cms/homepage')).toHaveLength(1),
    );
    expect(mock.calls('put', '/api/v1/cms/homepage')[0].body).toEqual({
        ...HOMEPAGE,
        heroHeading: 'Hello world',
    });
    expect(await screen.findByText('Saved.')).toBeInTheDocument();
});

test('saving the features PUTs the three blocks with the current hero', async () => {
    const { mock, user } = setup();
    const title = await screen.findByLabelText('Feature 2: title');
    await user.clear(title);
    await user.type(title, 'Libre');
    await user.click(screen.getByRole('button', { name: 'Save features' }));
    await waitFor(() =>
        expect(mock.calls('put', '/api/v1/cms/homepage')).toHaveLength(1),
    );
    expect(mock.calls('put', '/api/v1/cms/homepage')[0].body).toEqual({
        ...HOMEPAGE,
        features: [
            HOMEPAGE.features[0],
            { title: 'Libre', text: 'Free.' },
            HOMEPAGE.features[2],
        ],
    });
});

test('validates required fields and maps server field errors', async () => {
    const { mock, user } = setup();
    const heading = await screen.findByLabelText('Heading');
    await user.clear(heading);
    await user.click(screen.getByRole('button', { name: 'Save hero' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
        'This field is required.',
    );
    expect(mock.calls('put', '/api/v1/cms/homepage')).toHaveLength(0);

    mock.on(
        'put',
        '/api/v1/cms/homepage',
        422,
        problem(422, 'validation_failed', [
            { field: 'features[0].text', code: 'too_long' },
        ]),
    );
    await user.click(screen.getByRole('button', { name: 'Save features' }));
    const alert = await screen.findAllByRole('alert');
    expect(
        alert.some((a) => a.textContent?.includes('This is too long.')),
    ).toBe(true);
});

test('edits the terms in the Markdown editor with preview', async () => {
    const { mock, user } = setup();
    const terms = await screen.findByLabelText('Text');
    expect(terms).toHaveValue('Be nice.');
    await user.clear(terms);
    await user.type(terms, 'Be **very** nice.');
    await user.click(screen.getByRole('tab', { name: 'Preview' }));
    const preview = screen.getByRole('tabpanel', { name: 'Preview' });
    expect(within(preview).getByText('very').tagName).toBe('STRONG');
    await user.click(screen.getByRole('button', { name: 'Save terms' }));
    await waitFor(() =>
        expect(mock.calls('put', '/api/v1/cms/terms')).toHaveLength(1),
    );
    expect(mock.calls('put', '/api/v1/cms/terms')[0].body).toEqual({
        body: 'Be **very** nice.',
    });
});
