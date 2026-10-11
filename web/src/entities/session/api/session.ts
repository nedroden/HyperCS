import { ApiError, api, setCsrfToken, unwrap, type Schema } from '@/shared/api';

export type Me = Schema<'MeResponse'>;
export type Role = Schema<'Role'>;
export type LoginCredentials = Schema<'LoginRequest'>;
export type PreferencesUpdate = Schema<'UpdatePreferencesRequest'>;

/** `GET /me`: the account of the current session, or `null` for a guest (401). Also installs the CSRF token. */
export async function fetchMe(): Promise<Me | null> {
    try {
        const me = await unwrap(api.GET('/api/v1/me'));
        setCsrfToken(me.csrfToken);
        return me;
    } catch (error) {
        if (error instanceof ApiError && error.status === 401) {
            setCsrfToken(undefined);
            return null;
        }
        throw error;
    }
}

/** `POST /auth/login`. The session is rotated server-side; refetch `/me` afterwards (see `useSessionTransitions`). */
export function login(credentials: LoginCredentials): Promise<void> {
    return unwrap(api.POST('/api/v1/auth/login', { body: credentials }));
}

export function logout(): Promise<void> {
    return unwrap(api.POST('/api/v1/auth/logout'));
}

/** `PATCH /me/preferences` (language, time zone, notification and privacy flags). */
export function updatePreferences(update: PreferencesUpdate): Promise<void> {
    return unwrap(api.PATCH('/api/v1/me/preferences', { body: update }));
}
