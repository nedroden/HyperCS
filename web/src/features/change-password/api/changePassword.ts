import { api, unwrap, type Schema } from '@/shared/api';

export type ChangePasswordBody = Schema<'ChangePasswordRequest'>;

/** `PATCH /me/password`: the server revokes every other session of the account. */
export function changePassword(body: ChangePasswordBody): Promise<void> {
    return unwrap(api.PATCH('/api/v1/me/password', { body }));
}
