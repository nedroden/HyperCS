import type { ComponentProps } from 'react';
import ReactMarkdown from 'react-markdown';
import rehypeSanitize from 'rehype-sanitize';
import remarkGfm from 'remark-gfm';

const REMARK_PLUGINS = [remarkGfm];
const REHYPE_PLUGINS = [rehypeSanitize];

const components: ComponentProps<typeof ReactMarkdown>['components'] = {
    a: ({ node: _node, ...props }) => (
        <a {...props} target="_blank" rel="noopener nofollow ugc" />
    ),
    // Remote images in user content are not loaded (tracking, hot-linking); show a link instead.
    img: ({ node: _node, src, alt }) =>
        typeof src === 'string' ? (
            <a href={src} target="_blank" rel="noopener nofollow ugc">
                {alt || src}
            </a>
        ) : null,
};

/**
 * Renders user-written Markdown (GFM) safely: output is sanitized with `rehype-sanitize`, raw HTML in the
 * source is shown as text and never parsed, and links get `rel="noopener nofollow ugc"`.
 */
export function Markdown({ children }: { children: string }) {
    return (
        <div className="markdown">
            <ReactMarkdown
                remarkPlugins={REMARK_PLUGINS}
                rehypePlugins={REHYPE_PLUGINS}
                components={components}
            >
                {children}
            </ReactMarkdown>
        </div>
    );
}
