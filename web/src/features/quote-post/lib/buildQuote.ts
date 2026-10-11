/**
 * The Markdown inserted into the reply form when quoting a post: a blockquote with an attribution line,
 * followed by a blank line so the reply starts below it.
 */
export function buildQuote(authorName: string, body: string): string {
    const lines = body.replace(/\r\n?/g, '\n').trimEnd().split('\n');
    const quoted = lines.map((line) => (line ? `> ${line}` : '>')).join('\n');
    return `> **${authorName}**:\n>\n${quoted}\n\n`;
}
