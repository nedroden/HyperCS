import { QueryClientProvider, type QueryClient } from '@tanstack/react-query';
import type { ComponentType, ReactNode } from 'react';
import { BrowserRouter } from 'react-router-dom';
import { SessionProvider } from '@/entities/session';
import { createQueryClient } from '@/shared/api';

const defaultQueryClient = createQueryClient();

type ProvidersProps = {
    children: ReactNode;
    /** Tests pass their own client (no retries, fresh cache). */
    queryClient?: QueryClient;
    /** Tests pass a `MemoryRouter` wrapper. */
    Router?: ComponentType<{ children: ReactNode }>;
};

/** Query client, router and session state, in the order the rest of the app relies on. */
export function Providers({
    children,
    queryClient = defaultQueryClient,
    Router = BrowserRouter,
}: ProvidersProps) {
    return (
        <QueryClientProvider client={queryClient}>
            <Router>
                <SessionProvider>{children}</SessionProvider>
            </Router>
        </QueryClientProvider>
    );
}
