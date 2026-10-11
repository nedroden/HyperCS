import { render, screen } from '@testing-library/react';
import { expect, test, vi } from 'vitest';
import '@/shared/i18n';
import { ErrorBoundary } from './ErrorBoundary';

function Boom(): never {
    throw new Error('boom');
}

test('renders children when nothing fails', () => {
    render(
        <ErrorBoundary>
            <p>fine</p>
        </ErrorBoundary>,
    );
    expect(screen.getByText('fine')).toBeInTheDocument();
});

test('shows a fallback when a child throws', () => {
    const spy = vi.spyOn(console, 'error').mockImplementation(() => {});
    render(
        <ErrorBoundary>
            <Boom />
        </ErrorBoundary>,
    );
    expect(screen.getByRole('alert')).toHaveTextContent('Something went wrong');
    expect(
        screen.getByRole('button', { name: 'Reload page' }),
    ).toBeInTheDocument();
    spy.mockRestore();
});
