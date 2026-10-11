import { useTranslation } from 'react-i18next';
import type { ReactNode } from 'react';
import { Navigate, Outlet, useLocation } from 'react-router-dom';
import { useSession } from '@/entities/session';
import { loginRedirectState } from '@/shared/lib';
import { ErrorState, Loading } from '@/shared/ui';

/**
 * Layout route (renders `<Outlet />`, or `children` when given) that only shows content to logged-in users. Guests are sent to `/login` (which
 * returns them here afterwards); a failed session check shows an error with a retry. The API still
 * authorizes every request: this only saves the round trip and the confusing screen.
 */
export function RequireAuth({ children }: { children?: ReactNode } = {}) {
    const { t } = useTranslation();
    const { status, refresh } = useSession();
    const location = useLocation();
    switch (status) {
        case 'loading':
            return <Loading />;
        case 'error':
            return (
                <ErrorState
                    title={t('auth.sessionError.title')}
                    text={t('auth.sessionError.text')}
                    onRetry={() => void refresh()}
                />
            );
        case 'guest':
            return (
                <Navigate
                    to="/login"
                    replace
                    state={loginRedirectState(location)}
                />
            );
        case 'authenticated':
            return children ?? <Outlet />;
    }
}
