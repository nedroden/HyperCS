/** The 1-based page from a `?page=` value; anything that is not a positive integer becomes 1. */
export function pageFromParam(value: string | null): number {
    const page = Number(value);
    return Number.isInteger(page) && page >= 1 ? page : 1;
}
