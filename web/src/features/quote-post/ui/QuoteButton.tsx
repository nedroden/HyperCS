import { useTranslation } from 'react-i18next';
import type { Schema } from '@/shared/api';
import { buildQuote } from '../lib/buildQuote';

type QuoteButtonProps = {
    post: Pick<Schema<'Post'>, 'body' | 'author'>;
    /** Receives the Markdown blockquote to insert into the reply form. */
    onQuote: (markdown: string) => void;
};

/** "Quote" action of a post: hands a Markdown blockquote of the post to the reply form. */
export function QuoteButton({ post, onQuote }: QuoteButtonProps) {
    const { t } = useTranslation();
    return (
        <button
            type="button"
            className="link-button"
            onClick={() =>
                onQuote(buildQuote(post.author.displayName, post.body))
            }
        >
            {t('forum.quote')}
        </button>
    );
}
