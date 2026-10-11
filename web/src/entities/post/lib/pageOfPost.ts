/** The 1-based page of a topic a post lives on. */
export function pageOfPost(postNumber: number, perPage = 15): number {
    return Math.max(1, Math.ceil(postNumber / perPage));
}
