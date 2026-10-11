import { useTranslation } from 'react-i18next';
import { TopicRow, type TopicSummary } from '@/entities/topic';

/** The topics of a board in columns Topic / Replies / Views / Last post (pinned topics arrive first). */
export function TopicList({ topics }: { topics: TopicSummary[] }) {
    const { t } = useTranslation();
    return (
        <div
            className="topics"
            role="table"
            aria-label={t('forum.columns.topic')}
        >
            <div className="topics-head" role="row">
                <div role="columnheader">{t('forum.columns.topic')}</div>
                <div role="columnheader">{t('forum.columns.replies')}</div>
                <div role="columnheader">{t('forum.columns.views')}</div>
                <div role="columnheader">{t('forum.columns.lastPost')}</div>
            </div>
            {topics.map((topic) => (
                <TopicRow key={topic.id} topic={topic} />
            ))}
        </div>
    );
}
