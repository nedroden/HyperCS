import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { problemOf, queryKeys, useApiMutation } from '@/shared/api';
import { errorKey } from '@/shared/lib';
import {
    EmptyState,
    ErrorState,
    Loading,
    Panel,
    RelativeTime,
    translateError,
} from '@/shared/ui';
import { listSessions, revokeSession, type SessionInfo } from '../api/sessions';
import { describeDevice } from '../model/describeDevice';

/** The "Active sessions" settings section: signed-in devices with a revoke action (not for this device). */
export function SessionsPanel() {
    const { t, i18n } = useTranslation();
    const sessions = useQuery({
        queryKey: queryKeys.me.sessions(),
        queryFn: listSessions,
    });
    const [revoked, setRevoked] = useState(false);
    const revoke = useApiMutation({
        mutationFn: (id: string) => revokeSession(id),
        invalidates: [queryKeys.me.sessions()],
    });

    const deviceName = (s: SessionInfo) => {
        const { browser, os } = describeDevice(s.userAgent);
        if (browser && os)
            return t('settings.sessions.device', { browser, os });
        return browser ?? os ?? t('settings.sessions.unknownDevice');
    };

    const onRevoke = async (id: string) => {
        setRevoked(false);
        try {
            await revoke.mutateAsync(id);
            setRevoked(true);
        } catch {
            // Shown below from `revoke.error`.
        }
    };

    let body;
    if (sessions.isPending) {
        body = <Loading />;
    } else if (sessions.isError) {
        body = <ErrorState onRetry={() => void sessions.refetch()} />;
    } else if (sessions.data.length === 0) {
        body = <EmptyState text={t('settings.sessions.empty')} />;
    } else {
        body = (
            <table className="sessions-table">
                <thead>
                    <tr>
                        <th scope="col">{t('settings.sessions.deviceCol')}</th>
                        <th scope="col">{t('settings.sessions.created')}</th>
                        <th scope="col">{t('settings.sessions.lastActive')}</th>
                        <th scope="col">
                            <span className="sr-only">
                                {t('settings.sessions.actions')}
                            </span>
                        </th>
                    </tr>
                </thead>
                <tbody>
                    {sessions.data.map((s) => (
                        <tr key={s.id}>
                            <td>
                                {deviceName(s)}
                                {s.current && (
                                    <span className="current-tag">
                                        {t('settings.sessions.thisDevice')}
                                    </span>
                                )}
                            </td>
                            <td>
                                <RelativeTime date={s.createdAt} />
                            </td>
                            <td>
                                <RelativeTime date={s.lastSeenAt} />
                            </td>
                            <td>
                                {!s.current && (
                                    <button
                                        className="btn-link"
                                        type="button"
                                        disabled={revoke.isPending}
                                        aria-label={t(
                                            'settings.sessions.revokeLabel',
                                            { device: deviceName(s) },
                                        )}
                                        onClick={() => void onRevoke(s.id)}
                                    >
                                        {t('settings.sessions.revoke')}
                                    </button>
                                )}
                            </td>
                        </tr>
                    ))}
                </tbody>
            </table>
        );
    }

    return (
        <Panel
            id="sessions"
            title={t('settings.sessions.title')}
            intro={t('settings.sessions.intro')}
        >
            {revoke.isError && (
                <p className="field-error" role="alert">
                    {translateError(
                        t,
                        i18n,
                        errorKey(problemOf(revoke.error).code),
                    )}
                </p>
            )}
            {body}
            <p className="form-status" role="status">
                {revoked && t('settings.sessions.revoked')}
            </p>
        </Panel>
    );
}
