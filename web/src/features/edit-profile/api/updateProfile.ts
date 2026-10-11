import { api, unwrap, type Schema } from '@/shared/api';

export type ProfileUpdate = Schema<'UpdateProfileRequest'>;
export type AccountUpdate = Schema<'UpdateAccountRequest'>;

/** `PATCH /me/profile`: absent fields stay unchanged, `null` clears an optional field. */
export function updateProfile(body: ProfileUpdate): Promise<void> {
    return unwrap(api.PATCH('/api/v1/me/profile', { body }));
}

/** `PATCH /me/account` (first/last name, email). */
export function updateAccount(body: AccountUpdate): Promise<void> {
    return unwrap(api.PATCH('/api/v1/me/account', { body }));
}
