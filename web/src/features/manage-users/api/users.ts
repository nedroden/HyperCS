import { api, unwrap, type PageParams, type Schema } from '@/shared/api';

export type AdminUser = Schema<'AdminUser'>;
export type AdminUserPage = Schema<'AdminUserPage'>;
export type Role = Schema<'Role'>;

export const ROLES: readonly Role[] = ['admin', 'moderator', 'member'];

/** `GET /users` (admin): all accounts, oldest first. */
export function listUsers(params: PageParams = {}): Promise<AdminUserPage> {
    return unwrap(api.GET('/api/v1/users', { params: { query: params } }));
}

/** `PATCH /users/{username}/role` (admin). The last admin cannot be demoted (409 `last_admin`). */
export function updateRole(username: string, role: Role): Promise<void> {
    return unwrap(
        api.PATCH('/api/v1/users/{username}/role', {
            params: { path: { username } },
            body: { role },
        }),
    );
}
