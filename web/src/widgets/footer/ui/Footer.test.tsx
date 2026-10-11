import { screen } from '@testing-library/react';
import { afterEach, expect, test } from 'vitest';
import { i18n } from '@/shared/i18n';
import { renderApp } from '../../../../test/utils';
import { Footer } from './Footer';

afterEach(async () => {
    await i18n.changeLanguage('en-US');
});

test.each(['en-US', 'de'])(
    'shows the "Powered by" credit in English when the language is %s',
    async (language) => {
        await i18n.changeLanguage(language);
        renderApp(<Footer />);
        expect(await screen.findByText(/Powered by/)).toBeInTheDocument();
        expect(screen.getByRole('link', { name: 'HyperCS' })).toHaveAttribute(
            'href',
            'https://robertmonden.com',
        );
    },
);
