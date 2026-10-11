import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { expect, test, vi } from 'vitest';
import { EmptyState, ErrorState, Loading } from '.';

test('Loading is a polite status region', () => {
    render(<Loading />);
    expect(screen.getByRole('status')).toHaveTextContent('Loading…');
});

test('EmptyState shows defaults, overrides and an action', () => {
    const { rerender } = render(<EmptyState />);
    expect(screen.getByText('Nothing here yet')).toBeInTheDocument();
    rerender(
        <EmptyState
            title="No topics"
            text="Be the first."
            action={<a href="/new">New topic</a>}
        />,
    );
    expect(screen.getByText('No topics')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'New topic' })).toBeInTheDocument();
});

test('ErrorState is an alert with a working retry button', async () => {
    const onRetry = vi.fn();
    render(<ErrorState onRetry={onRetry} />);
    expect(screen.getByRole('alert')).toHaveTextContent('Something went wrong');
    await userEvent.click(screen.getByRole('button', { name: 'Try again' }));
    expect(onRetry).toHaveBeenCalledOnce();
});

test('ErrorState without retry renders no button', () => {
    render(<ErrorState />);
    expect(screen.queryByRole('button')).toBeNull();
});
