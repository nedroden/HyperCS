import { screen } from '@testing-library/react';
import { expect, test } from 'vitest';
import { mockApi, problem, renderApp } from '../../../../test/utils';
import { TermsPage } from './TermsPage';

test('renders the Markdown terms text, sanitized', async () => {
    const mock = mockApi();
    mock.on('get', '/api/v1/cms/terms', 200, {
        body: '## Be nice\n\nRead the **rules**.\n\n<script>alert(1)</script>',
    });
    const { container } = renderApp(<TermsPage />, { mock });
    expect(
        screen.getByRole('heading', { name: 'Terms of service' }),
    ).toBeInTheDocument();
    expect(
        await screen.findByRole('heading', { name: 'Be nice' }),
    ).toBeInTheDocument();
    expect(screen.getByText('rules').tagName).toBe('STRONG');
    expect(container.querySelector('script')).toBeNull();
});

test('shows an error state when the terms cannot be loaded', async () => {
    const mock = mockApi();
    mock.onUnlisted(
        'get',
        '/api/v1/cms/terms',
        500,
        problem(500, 'internal_error'),
    );
    renderApp(<TermsPage />, { mock });
    expect(
        await screen.findByText('The terms of service could not be loaded.'),
    ).toBeInTheDocument();
});
