import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { expect, test } from 'vitest';
import { i18n } from '@/shared/i18n';
import { MarkdownEditor } from './MarkdownEditor';

function Harness({ maxLength }: { maxLength?: number }) {
    const [value, setValue] = useState('');
    return (
        <>
            <label htmlFor="body">Message</label>
            <MarkdownEditor
                id="body"
                value={value}
                onChange={setValue}
                maxLength={maxLength}
            />
        </>
    );
}

test('updates the character counter while typing', async () => {
    render(<Harness />);
    expect(screen.getByText('0 / 10000 characters')).toBeInTheDocument();
    await userEvent.type(screen.getByLabelText('Message'), 'abc');
    expect(screen.getByText('3 / 10000 characters')).toBeInTheDocument();
});

test('enforces the configured maximum length', async () => {
    render(<Harness maxLength={5} />);
    await userEvent.type(screen.getByLabelText('Message'), 'abcdefgh');
    expect(screen.getByLabelText('Message')).toHaveValue('abcde');
    expect(screen.getByText('5 / 5 characters')).toBeInTheDocument();
});

test('uses the singular form for one character', async () => {
    render(<Harness />);
    await userEvent.type(screen.getByLabelText('Message'), 'a');
    expect(screen.getByText('1 / 10000 character')).toBeInTheDocument();
});

test('preview tab renders sanitized markdown and write tab returns', async () => {
    render(<Harness />);
    await userEvent.type(
        screen.getByLabelText('Message'),
        '**bold** <script>x</script>',
    );
    await userEvent.click(screen.getByRole('tab', { name: 'Preview' }));
    expect(screen.getByRole('tab', { name: 'Preview' })).toHaveAttribute(
        'aria-selected',
        'true',
    );
    const panel = screen.getByRole('tabpanel', { name: 'Preview' });
    expect(panel.querySelector('strong')).toHaveTextContent('bold');
    expect(panel.querySelector('script')).toBeNull();
    await userEvent.click(screen.getByRole('tab', { name: 'Write' }));
    expect(screen.getByLabelText('Message')).toBeVisible();
});

test('shows a hint when there is nothing to preview', async () => {
    render(<Harness />);
    await userEvent.click(screen.getByRole('tab', { name: 'Preview' }));
    expect(screen.getByText('Nothing to preview.')).toBeInTheDocument();
});

test('tabs are keyboard operable with arrow keys', async () => {
    render(<Harness />);
    screen.getByRole('tab', { name: 'Write' }).focus();
    await userEvent.keyboard('{ArrowRight}');
    expect(screen.getByRole('tab', { name: 'Preview' })).toHaveFocus();
    expect(screen.getByRole('tab', { name: 'Preview' })).toHaveAttribute(
        'aria-selected',
        'true',
    );
    await userEvent.keyboard('{ArrowLeft}');
    expect(screen.getByRole('tab', { name: 'Write' })).toHaveFocus();
});

test('counter is localized', async () => {
    await i18n.changeLanguage('de');
    render(<Harness />);
    expect(screen.getByText('0 / 10000 Zeichen')).toBeInTheDocument();
    await i18n.changeLanguage('en-US');
});
