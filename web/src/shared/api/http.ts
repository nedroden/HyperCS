/** Base path of the HyperCS API (proxied to the backend by Vite in development). */
export const API_BASE = '/api/v1';

export async function getJson<T>(path: string): Promise<T> {
    const res = await fetch(`${API_BASE}${path}`);
    if (!res.ok) throw new Error(`Request to ${path} failed: ${res.status}`);
    return (await res.json()) as T;
}
