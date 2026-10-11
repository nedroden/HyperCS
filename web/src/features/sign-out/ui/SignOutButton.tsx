import { useNavigate } from 'react-router-dom';
import { logout, useSessionTransitions } from '@/entities/session';
import { useApiMutation } from '@/shared/api';

type SignOutButtonProps = {
    /** Visible text, e.g. `t('userbox.logout')`. */
    label: string;
    className?: string;
};

/**
 * Logs out (`POST /auth/logout`), turns the session into a guest and goes to the homepage. Even if the
 * request fails the local state is reset: the server session expires on its own.
 */
export function SignOutButton({
    label,
    className = 'link-button',
}: SignOutButtonProps) {
    const navigate = useNavigate();
    const { afterLogout } = useSessionTransitions();
    const signOut = useApiMutation({
        mutationFn: logout,
        onSettled: () => {
            afterLogout();
            navigate('/');
        },
    });
    return (
        <button
            type="button"
            className={className}
            disabled={signOut.isPending}
            onClick={() => signOut.mutate()}
        >
            {label}
        </button>
    );
}
