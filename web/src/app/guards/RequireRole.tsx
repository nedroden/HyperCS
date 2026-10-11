import { useTranslation } from 'react-i18next';
import { Link, Outlet } from 'react-router-dom';
import { useSession } from '@/entities/session';
import { ErrorState } from '@/shared/ui';
import { RequireAuth } from './RequireAuth';

type RequireRoleProps = {
    /** `admin`: admins only. `moderator`: moderators and admins. */
    role: 'admin' | 'moderator';
};

function RoleGate({ role }: RequireRoleProps) {
    const { t } = useTranslation();
    const { isAdmin, isModerator } = useSession();
    if (role === 'admin' ? isAdmin : isModerator) return <Outlet />;
    return (
        <ErrorState
            title={t('auth.forbidden.title')}
            text={t('auth.forbidden.text')}
            action={
                <Link className="btn" to="/">
                    {t('common.backHome')}
                </Link>
            }
        />
    );
}

/**
 * Layout route for role-restricted screens: guests go to login (see `RequireAuth`), members without the
 * role see an "access denied" state. The API remains the real gate.
 */
export function RequireRole({ role }: RequireRoleProps) {
    return (
        <RequireAuth>
            <RoleGate role={role} />
        </RequireAuth>
    );
}
