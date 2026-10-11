import { api, unwrap, type Schema } from '@/shared/api';

export type RegisterRequest = Schema<'RegisterRequest'>;
export type RegisteredUser = Schema<'UserSummary'>;

/** `POST /auth/register`: creates a member account (201); it does not log the user in. */
export function register(body: RegisterRequest): Promise<RegisteredUser> {
    return unwrap(api.POST('/api/v1/auth/register', { body }));
}
