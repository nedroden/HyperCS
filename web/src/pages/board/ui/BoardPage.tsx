import { useTranslation } from 'react-i18next';
import { Link } from 'react-router-dom';
import { useRelativeTime } from '@/shared/lib';
import { Breadcrumbs, Pagination } from '@/shared/ui';
import { board, topics } from '../model/placeholder';

export function BoardPage() {
    const { t } = useTranslation();
    const ago = useRelativeTime();
    const when = (days: number) => (days === 0 ? ago(3, 'hour') : ago(days));

    return (
        <>
            <Breadcrumbs
                items={[
                    { label: t('forum.boardIndex'), to: '/forum' },
                    { label: board.category },
                    { label: board.name },
                ]}
            />

            <div className="title-row">
                <div>
                    <h2 className="page-title">{board.name}</h2>
                    <p>{board.description}</p>
                </div>
                <a className="btn" href="#">
                    {t('forum.newTopic')}
                </a>
            </div>

            <div className="topics">
                <div className="topics-head">
                    <div>{t('forum.columns.topic')}</div>
                    <div>{t('forum.columns.replies')}</div>
                    <div>{t('forum.columns.views')}</div>
                    <div>{t('forum.columns.lastPost')}</div>
                </div>
                {topics.map((topic) => (
                    <div className="topic" key={topic.title}>
                        <div>
                            {topic.pinned && (
                                <span className="tag">{t('forum.pinned')}</span>
                            )}
                            {topic.locked && (
                                <span className="tag locked">
                                    {t('forum.locked')}
                                </span>
                            )}
                            <Link
                                className="title"
                                to={`/forum/topics/${topic.id}`}
                            >
                                {topic.title}
                            </Link>
                            <div className="meta">
                                {t('forum.by')} <a href="#">{topic.starter}</a>{' '}
                                &middot; {when(topic.startedDays)}
                            </div>
                        </div>
                        <div className="num">{topic.replies}</div>
                        <div className="num">{topic.views}</div>
                        <div className="last-post">
                            {t('forum.by')} <a href="#">{topic.lastPoster}</a>
                            <br />
                            {when(topic.lastDays)}
                        </div>
                    </div>
                ))}
            </div>

            <Pagination current={1} total={3} />
        </>
    );
}
