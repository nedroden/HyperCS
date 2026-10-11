/** The entries of `values` that differ from `initial` (shallow `!==`): the body of a partial PATCH. */
export function changedFields<T extends Record<string, unknown>>(
    values: T,
    initial: T,
): Partial<T> {
    const changed: Partial<T> = {};
    for (const key of Object.keys(values) as (keyof T)[]) {
        if (values[key] !== initial[key]) changed[key] = values[key];
    }
    return changed;
}
