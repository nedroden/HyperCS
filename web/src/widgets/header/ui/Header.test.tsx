import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import { expect, test } from 'vitest';
import { SessionProvider, useSession } from '@/entities/session';
import '@/shared/i18n';
import { Header } from './Header';

function LoginButton() {
    const { login } = useSession();
    return <button onClick={login}>test-login</button>;
}

test('switches between logged-out and logged-in header', async () => {
    render(
        <MemoryRouter>
            <SessionProvider>
                <Header />
                <LoginButton />
            </SessionProvider>
        </MemoryRouter>,
    );
    expect(screen.getAllByText('Login').length).toBeGreaterThan(0);
    await userEvent.click(screen.getByText('test-login'));
    expect(screen.getByText('Administrator')).toBeInTheDocument();
    expect(screen.getAllByText('Logout').length).toBe(2);
});
