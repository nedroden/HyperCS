/** Returns the URL if it is a plain `http(s)` URL (safe to put in an `href`), otherwise `null`. */
export function safeWebUrl(value: string | null | undefined): URL | null {
    if (!value) return null;
    try {
        const url = new URL(value);
        return url.protocol === 'http:' || url.protocol === 'https:'
            ? url
            : null;
    } catch {
        return null;
    }
}
