import { api, unwrap, type Schema } from '@/shared/api';

export type SessionInfo = Schema<'SessionInfo'>;

/** `GET /me/sessions`: the account's signed-in devices (not paginated). */
export function listSessions(): Promise<SessionInfo[]> {
    return unwrap(api.GET('/api/v1/me/sessions'));
}

/** `DELETE /me/sessions/{id}`: signs that device out. */
export function revokeSession(id: string): Promise<void> {
    return unwrap(
        api.DELETE('/api/v1/me/sessions/{id}', { params: { path: { id } } }),
    );
}
