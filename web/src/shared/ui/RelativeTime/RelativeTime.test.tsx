import { render, screen } from '@testing-library/react';
import { afterEach, expect, test } from 'vitest';
import { i18n } from '@/shared/i18n';
import { TimeZoneProvider } from '@/shared/lib';
import { RelativeTime } from './RelativeTime';

const NOW = new Date('2026-10-07T12:00:00Z');

afterEach(() => i18n.changeLanguage('en-US'));

test('shows a relative time with an absolute UTC tooltip by default', () => {
    render(<RelativeTime date="2026-09-30T12:00:00Z" now={NOW} />);
    const el = screen.getByText('7 days ago');
    expect(el.tagName).toBe('TIME');
    expect(el).toHaveAttribute('datetime', '2026-09-30T12:00:00.000Z');
    expect(el.getAttribute('title')).toContain('September 30, 2026');
    expect(el.getAttribute('title')).toContain('12:00');
});

test('formats the tooltip in the provided time zone', () => {
    render(
        <TimeZoneProvider timeZone="Asia/Tokyo">
            <RelativeTime date="2026-09-30T12:00:00Z" now={NOW} />
        </TimeZoneProvider>,
    );
    expect(screen.getByText('7 days ago').getAttribute('title')).toContain(
        '9:00',
    );
});

test('is localized', async () => {
    await i18n.changeLanguage('de');
    render(<RelativeTime date="2026-10-04T12:00:00Z" now={NOW} />);
    expect(screen.getByText('vor 3 Tagen')).toBeInTheDocument();
});

test('picks sensible units and handles the present and future', () => {
    render(
        <>
            <RelativeTime date="2026-10-07T11:59:40Z" now={NOW} />
            <RelativeTime date="2026-10-07T09:00:00Z" now={NOW} />
            <RelativeTime date="2026-10-07T12:30:00Z" now={NOW} />
        </>,
    );
    expect(screen.getByText('now')).toBeInTheDocument();
    expect(screen.getByText('3 hours ago')).toBeInTheDocument();
    expect(screen.getByText('in 30 minutes')).toBeInTheDocument();
});

test('renders nothing for an invalid date', () => {
    const { container } = render(<RelativeTime date="nope" now={NOW} />);
    expect(container).toBeEmptyDOMElement();
});
