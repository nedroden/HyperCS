import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type { ReactNode } from 'react';
import { BrowserRouter } from 'react-router-dom';
import { SessionProvider } from '@/entities/session';

const queryClient = new QueryClient();

export function Providers({ children }: { children: ReactNode }) {
    return (
        <QueryClientProvider client={queryClient}>
            <SessionProvider>
                <BrowserRouter>{children}</BrowserRouter>
            </SessionProvider>
        </QueryClientProvider>
    );
}
