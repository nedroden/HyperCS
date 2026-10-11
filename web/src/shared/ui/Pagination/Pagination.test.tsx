import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter, useLocation } from 'react-router-dom';
import { expect, test } from 'vitest';
import { Pagination } from './Pagination';

function Where() {
    const { search } = useLocation();
    return <output>{search}</output>;
}

function setup(current: number, total: number) {
    return render(
        <MemoryRouter>
            <Pagination current={current} total={total} />
            <Where />
        </MemoryRouter>,
    );
}

test('marks the current page with aria-current and links the others', () => {
    setup(2, 3);
    expect(screen.getByText('2')).toHaveAttribute('aria-current', 'page');
    expect(screen.getByRole('link', { name: 'Page 1' })).toHaveAttribute(
        'href',
        '/?page=1',
    );
    expect(screen.getByRole('link', { name: 'Page 3' })).toBeInTheDocument();
    expect(
        screen.getByRole('link', { name: 'Previous page' }),
    ).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Next page' })).toBeInTheDocument();
});

test('has no previous link on the first page and no next link on the last', () => {
    const { unmount } = setup(1, 3);
    expect(screen.queryByRole('link', { name: 'Previous page' })).toBeNull();
    unmount();
    setup(3, 3);
    expect(screen.queryByRole('link', { name: 'Next page' })).toBeNull();
});

test('renders nothing for a single page', () => {
    const { container } = setup(1, 1);
    expect(container.querySelector('nav')).toBeNull();
});

test('is keyboard operable', async () => {
    setup(1, 3);
    await userEvent.tab();
    expect(screen.getByRole('link', { name: 'Page 2' })).toHaveFocus();
    await userEvent.keyboard('{Enter}');
    expect(screen.getByRole('status')).toHaveTextContent('?page=2');
});

test('collapses long ranges with ellipses', () => {
    setup(10, 20);
    expect(screen.getByText('10')).toHaveAttribute('aria-current', 'page');
    expect(screen.getByRole('link', { name: 'Page 1' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Page 20' })).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: 'Page 5' })).toBeNull();
    expect(screen.getAllByText('…')).toHaveLength(2);
});
