import { useId, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Markdown, RelativeTime } from '@/shared/ui';
import { excerptOf } from '../lib/excerpt';
import type { NewsItem } from '../api/news';

/** One entry of the "Latest news" list (mocks/home.html): title, relative date and an excerpt that expands. */
export function NewsItemCard({ item }: { item: NewsItem }) {
    const { t } = useTranslation();
    const [open, setOpen] = useState(false);
    const bodyId = useId();
    const { text, truncated } = excerptOf(item.body);
    return (
        <article>
            <h4>{item.title}</h4>
            <div className="date">
                <RelativeTime date={item.publishedAt} />
            </div>
            <div id={bodyId}>
                {open ? <Markdown>{item.body}</Markdown> : <p>{text}</p>}
            </div>
            {truncated && (
                <button
                    type="button"
                    className="news-toggle"
                    aria-expanded={open}
                    aria-controls={bodyId}
                    onClick={() => setOpen(!open)}
                >
                    {open ? t('news.showLess') : t('news.readMore')}
                </button>
            )}
        </article>
    );
}
