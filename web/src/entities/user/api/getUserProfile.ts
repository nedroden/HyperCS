import { api, unwrap, type Schema } from '@/shared/api';

export type UserProfile = Schema<'PublicProfile'>;

/** `GET /users/{username}`: the public profile; rejects with a 404 `ApiError` for an unknown username. */
export function getUserProfile(username: string): Promise<UserProfile> {
    return unwrap(
        api.GET('/api/v1/users/{username}', {
            params: { path: { username } },
        }),
    );
}
