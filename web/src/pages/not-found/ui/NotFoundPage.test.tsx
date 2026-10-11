import { screen, waitFor } from '@testing-library/react';
import { expect, test } from 'vitest';
import { mockApi, renderApp } from '../../../../test/utils';
import { NotFoundPage } from './NotFoundPage';

test('explains the page is missing and links home', () => {
    renderApp(<NotFoundPage />);
    expect(
        screen.getByRole('heading', {
            name: 'This page could not be found.',
        }),
    ).toBeInTheDocument();
    expect(
        screen.getByRole('link', { name: 'Back to the homepage' }),
    ).toHaveAttribute('href', '/');
    expect(document.title).toBe('Page not found – HyperCS');
});

test('the tab title uses the configured site title', async () => {
    const mock = mockApi();
    mock.on('get', '/api/v1/cms/site', 200, {
        siteTitle: 'My Club',
        slogan: 's',
        defaultLanguage: 'en-US',
    });
    renderApp(<NotFoundPage />, { mock });
    await waitFor(() =>
        expect(document.title).toBe('Page not found – My Club'),
    );
});
