import { useTranslation } from 'react-i18next';
import { Link } from 'react-router-dom';
import { topicPath, userPath } from '@/shared/lib';
import { RelativeTime } from '@/shared/ui';
import type { TopicSummary } from '../api/getTopicPage';
import { TopicBadges } from './TopicBadges';

/** One row of a board's topic list (mocks/board.html). */
export function TopicRow({ topic }: { topic: TopicSummary }) {
    const { t } = useTranslation();
    return (
        <div className="topic" role="row">
            <div role="cell">
                <TopicBadges pinned={topic.pinned} locked={topic.locked} />
                <Link className="title" to={topicPath(topic.id, topic.slug)}>
                    {topic.title}
                </Link>
                <div className="meta">
                    {t('forum.by')}{' '}
                    <Link to={userPath(topic.author.username)}>
                        {topic.author.displayName}
                    </Link>{' '}
                    &middot; <RelativeTime date={topic.createdAt} />
                </div>
            </div>
            <div className="num" role="cell">
                {topic.replyCount}
            </div>
            <div className="num" role="cell">
                {topic.viewCount}
            </div>
            <div className="last-post" role="cell">
                {topic.lastPost && (
                    <>
                        {t('forum.by')}{' '}
                        <Link to={userPath(topic.lastPost.author.username)}>
                            {topic.lastPost.author.displayName}
                        </Link>
                        <br />
                        <RelativeTime date={topic.lastPost.createdAt} />
                    </>
                )}
            </div>
        </div>
    );
}
