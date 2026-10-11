import { screen } from '@testing-library/react';
import { expect, test } from 'vitest';
import { makeMe, renderApp } from '../../../../test/utils';
import { Header } from './Header';

test('admins get an Admin link in the user box; members and guests do not', async () => {
    const admin = renderApp(<Header />, { me: makeMe({ role: 'admin' }) });
    expect(await screen.findByRole('link', { name: 'Admin' })).toHaveAttribute(
        'href',
        '/admin',
    );
    admin.unmount();

    renderApp(<Header />, { me: makeMe() });
    await screen.findByRole('link', { name: 'Alice' });
    expect(screen.queryByRole('link', { name: 'Admin' })).toBeNull();
});

test('guests have no Admin link', async () => {
    renderApp(<Header />);
    await screen.findByRole('link', { name: 'Register' });
    expect(screen.queryByRole('link', { name: 'Admin' })).toBeNull();
});
