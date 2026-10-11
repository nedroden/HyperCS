/** Maximum length of the plain-text excerpt shown in news lists. */
export const EXCERPT_LENGTH = 200;

/** Plain text of a Markdown body with the most common syntax removed, whitespace collapsed. */
export function plainText(markdown: string): string {
    return markdown
        .replace(/```[\s\S]*?```/g, ' ')
        .replace(/!?\[([^\]]*)\]\([^)]*\)/g, '$1')
        .replace(/^\s{0,3}(#{1,6}|>|[-*+]|\d+\.)\s+/gm, '')
        .replace(/[*_`~]/g, '')
        .replace(/\s+/g, ' ')
        .trim();
}

/** Short excerpt of a news body; `truncated` tells whether there is more to read. */
export function excerptOf(
    markdown: string,
    length = EXCERPT_LENGTH,
): { text: string; truncated: boolean } {
    const text = plainText(markdown);
    if (text.length <= length) {
        return { text, truncated: text !== markdown.trim() };
    }
    const cut = text.slice(0, length);
    const space = cut.lastIndexOf(' ');
    return {
        text: `${(space > length / 2 ? cut.slice(0, space) : cut).trimEnd()}…`,
        truncated: true,
    };
}
