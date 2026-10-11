import type { ReactNode } from 'react';
import { useTranslation } from 'react-i18next';
import { Link } from 'react-router-dom';
import type { Schema } from '@/shared/api';
import { topicPath, userPath } from '@/shared/lib';
import { Avatar, Markdown, RelativeTime } from '@/shared/ui';

type PostCardProps = {
    post: Schema<'Post'>;
    /** Heading of the post: the topic title for the first post, "Re: ..." for replies (derived by the caller). */
    title: string;
    /** Permalink target, i.e. the topic page that contains the post. */
    permalink: { topicId: string; slug?: string; page: number };
    /** Action links/buttons (quote, edit, ...); only the ones the viewer may use. */
    actions?: ReactNode;
};

/** A post with its author panel (mocks/topic.html). Markdown is rendered sanitized. */
export function PostCard({ post, title, permalink, actions }: PostCardProps) {
    const { t } = useTranslation();
    const { author } = post;
    return (
        <article className="post" id={`post-${post.id}`}>
            <div className="post-author">
                <Avatar name={author.displayName} size="lg" />
                <div className="name">
                    <Link to={userPath(author.username)}>
                        {author.displayName}
                    </Link>
                </div>
                {author.role !== 'member' && (
                    <div className="role">{t(`forum.role.${author.role}`)}</div>
                )}
                <div>{t('forum.postCount', { count: author.postCount })}</div>
                <div>
                    {t('forum.joined')} <RelativeTime date={author.joinedAt} />
                </div>
            </div>
            <div className="post-body">
                <div className="post-meta">
                    <span>{title}</span>
                    <span>
                        <RelativeTime date={post.createdAt} /> &middot;{' '}
                        <Link
                            to={topicPath(permalink.topicId, permalink.slug, {
                                page: permalink.page,
                                postId: post.id,
                            })}
                            aria-label={t('forum.permalink', {
                                number: post.number,
                            })}
                        >
                            #{post.number}
                        </Link>
                    </span>
                </div>
                <div className="post-content">
                    <Markdown>{post.body}</Markdown>
                </div>
                {post.editedAt && (
                    <div className="post-edited">
                        {post.editedBy
                            ? t('forum.editedBy', {
                                  name: post.editedBy.displayName,
                              })
                            : t('forum.edited')}{' '}
                        <RelativeTime date={post.editedAt} />
                    </div>
                )}
                {actions && <div className="post-actions">{actions}</div>}
            </div>
        </article>
    );
}
