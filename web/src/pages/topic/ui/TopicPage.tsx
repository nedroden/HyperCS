import { useTranslation } from 'react-i18next';
import { useRelativeTime } from '@/shared/lib';
import { Avatar, Breadcrumbs, Pagination } from '@/shared/ui';
import { posts, topic } from '../model/placeholder';

export function TopicPage() {
    const { t } = useTranslation();
    const ago = useRelativeTime();

    return (
        <>
            <Breadcrumbs
                items={[
                    { label: t('forum.boardIndex'), to: '/forum' },
                    { label: topic.category },
                    {
                        label: topic.boardName,
                        to: `/forum/boards/${topic.boardId}`,
                    },
                    { label: topic.title },
                ]}
            />

            <div className="title-row">
                <div>
                    <h2 className="page-title">{topic.title}</h2>
                    <p>
                        {t('forum.startedBy')} <a href="#">{topic.starter}</a>{' '}
                        &middot; {ago(topic.startedDays)} &middot;{' '}
                        {t('forum.posts', { count: posts.length })}
                    </p>
                </div>
                <a className="btn" href="#reply">
                    {t('forum.postReply')}
                </a>
            </div>

            <div className="posts">
                {posts.map((post) => (
                    <article className="post" key={post.id}>
                        <div className="post-author">
                            <Avatar name={post.author} size="lg" />
                            <div className="name">
                                <a href="#">{post.author}</a>
                            </div>
                            {post.admin && (
                                <div className="role">{t('forum.admin')}</div>
                            )}
                            <div>
                                {t('forum.postCount', {
                                    count: post.postCount,
                                })}
                            </div>
                            <div>
                                {t('forum.joined', {
                                    when: ago(post.joinedDays),
                                })}
                            </div>
                        </div>
                        <div className="post-body">
                            <div className="post-meta">
                                <span>{post.title}</span>
                                <span>
                                    {ago(post.days)} &middot;{' '}
                                    <a href="#">#{post.id}</a>
                                </span>
                            </div>
                            <div className="post-content">
                                {post.paragraphs.map((p) => (
                                    <p key={p}>{p}</p>
                                ))}
                            </div>
                            <div className="post-actions">
                                <a href="#">{t('forum.quote')}</a>
                                {post.canModerate ? (
                                    <>
                                        <a href="#">{t('forum.edit')}</a>
                                        <a href="#">{t('forum.delete')}</a>
                                    </>
                                ) : (
                                    <a href="#">{t('forum.report')}</a>
                                )}
                            </div>
                        </div>
                    </article>
                ))}
            </div>

            <Pagination current={1} total={2} />

            <section className="reply" id="reply">
                <h3>{t('forum.replyHeading')}</h3>
                <textarea placeholder={t('forum.replyPlaceholder')} />
                <button className="btn" type="button">
                    {t('forum.submit')}
                </button>
            </section>
        </>
    );
}
