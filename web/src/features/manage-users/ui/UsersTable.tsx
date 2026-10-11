import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSearchParams } from 'react-router-dom';
import { useSession } from '@/entities/session';
import { problemOf } from '@/shared/api';
import { errorKey } from '@/shared/lib';
import {
    EmptyState,
    ErrorState,
    Loading,
    Pagination,
    RelativeTime,
    translateError,
} from '@/shared/ui';
import { ROLES, type Role } from '../api/users';
import { useUpdateRole, useUsers } from '../model/useUsers';

function currentPage(params: URLSearchParams): number {
    const page = Number(params.get('page'));
    return Number.isInteger(page) && page >= 1 ? page : 1;
}

/** Paginated user list with a role dropdown per row (mocks/admin-users.html). */
export function UsersTable() {
    const { t, i18n } = useTranslation();
    const { user: me } = useSession();
    const [params] = useSearchParams();
    const page = currentPage(params);
    const users = useUsers({ page });
    const updateRole = useUpdateRole();
    const [notice, setNotice] = useState<{
        kind: 'error' | 'ok';
        text: string;
    } | null>(null);

    const change = async (username: string, name: string, role: Role) => {
        setNotice(null);
        try {
            await updateRole.mutateAsync({ username, role });
            setNotice({
                kind: 'ok',
                text: t('admin.users.roleChanged', {
                    name,
                    role: t(`admin.roles.${role}`),
                }),
            });
        } catch (error) {
            setNotice({
                kind: 'error',
                text: `${t('admin.users.changeFailed', { name })} ${translateError(
                    t,
                    i18n,
                    errorKey(problemOf(error).code),
                )}`,
            });
        }
    };

    if (users.isPending) return <Loading />;
    if (users.isError)
        return <ErrorState onRetry={() => void users.refetch()} />;
    const { items, totalPages } = users.data;
    if (items.length === 0) return <EmptyState text={t('admin.users.empty')} />;

    return (
        <>
            {notice && (
                <p
                    className={
                        notice.kind === 'error'
                            ? 'admin-alert'
                            : 'admin-alert admin-alert-ok'
                    }
                    role={notice.kind === 'error' ? 'alert' : 'status'}
                >
                    {notice.text}
                </p>
            )}
            <div className="table-scroll">
                <table className="admin-table">
                    <thead>
                        <tr>
                            <th scope="col">{t('admin.users.username')}</th>
                            <th scope="col">{t('admin.users.displayName')}</th>
                            <th scope="col">{t('admin.users.email')}</th>
                            <th scope="col">{t('admin.users.joined')}</th>
                            <th scope="col">{t('admin.users.role')}</th>
                        </tr>
                    </thead>
                    <tbody>
                        {items.map((user) => {
                            const own = user.id === me?.id;
                            const hintId = `own-role-${user.id}`;
                            return (
                                <tr key={user.id}>
                                    <td>
                                        {user.username}{' '}
                                        {own && (
                                            <span className="muted">
                                                {t('admin.users.you')}
                                            </span>
                                        )}
                                    </td>
                                    <td>{user.displayName}</td>
                                    <td>{user.email}</td>
                                    <td>
                                        <RelativeTime date={user.createdAt} />
                                    </td>
                                    <td>
                                        <select
                                            aria-label={t(
                                                'admin.users.roleFor',
                                                {
                                                    name: user.username,
                                                },
                                            )}
                                            value={user.role}
                                            disabled={
                                                own || updateRole.isPending
                                            }
                                            aria-describedby={
                                                own ? hintId : undefined
                                            }
                                            onChange={(e) =>
                                                void change(
                                                    user.username,
                                                    user.displayName,
                                                    e.target.value as Role,
                                                )
                                            }
                                        >
                                            {ROLES.map((role) => (
                                                <option key={role} value={role}>
                                                    {t(`admin.roles.${role}`)}
                                                </option>
                                            ))}
                                        </select>
                                        {own && (
                                            <span
                                                className="sr-only"
                                                id={hintId}
                                            >
                                                {t('admin.users.ownRole')}
                                            </span>
                                        )}
                                    </td>
                                </tr>
                            );
                        })}
                    </tbody>
                </table>
            </div>
            {totalPages > 1 && <Pagination current={page} total={totalPages} />}
        </>
    );
}
