import type { Location } from 'react-router-dom';

/** Router state passed to `/login` by the route guards so the login page can return to where the user was. */
export type LoginRedirectState = { from: string };

export function loginRedirectState(
    location: Pick<Location, 'pathname' | 'search' | 'hash'>,
): LoginRedirectState {
    return { from: `${location.pathname}${location.search}${location.hash}` };
}

/** The post-login destination from router state; only same-origin absolute paths are accepted, else `/`. */
export function redirectTargetFrom(state: unknown): string {
    const from =
        state && typeof state === 'object' && 'from' in state
            ? (state as { from: unknown }).from
            : undefined;
    return typeof from === 'string' &&
        from.startsWith('/') &&
        !from.startsWith('//') &&
        !from.startsWith('/\\')
        ? from
        : '/';
}

export function hasLoginRedirect(state: unknown): boolean {
    return redirectTargetFrom(state) !== '/';
}
