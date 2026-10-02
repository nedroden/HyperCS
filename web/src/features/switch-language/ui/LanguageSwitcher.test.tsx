import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nextProvider } from 'react-i18next';
import { afterEach, expect, test } from 'vitest';
import { i18n } from '@/shared/i18n';
import { LanguageSwitcher } from './LanguageSwitcher';

afterEach(() => i18n.changeLanguage('en-US'));

test('switches the language to German', async () => {
    render(
        <I18nextProvider i18n={i18n}>
            <LanguageSwitcher />
        </I18nextProvider>,
    );
    await userEvent.click(
        screen.getByText('English (US)', { selector: 'summary' }),
    );
    await userEvent.click(screen.getByRole('button', { name: 'Deutsch' }));
    expect(i18n.language).toBe('de');
    expect(document.documentElement.lang).toBe('de');
});
