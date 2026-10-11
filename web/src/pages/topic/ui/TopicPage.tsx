import { useEffect, useState } from 'react';
import { Trans, useTranslation } from 'react-i18next';
import {
    Link,
    Navigate,
    useLocation,
    useNavigate,
    useParams,
    useSearchParams,
} from 'react-router-dom';
import { pageOfPost } from '@/entities/post';
import { useSession } from '@/entities/session';
import {
    canModerateTopic,
    canReplyToTopic,
    TopicBadges,
    useTopicPage,
} from '@/entities/topic';
import { ModerationBar } from '@/features/moderate-topic';
import { MoveTopicButton } from '@/features/move-topic';
import { ReplyForm, type ReplyInsertion } from '@/features/reply-to-topic';
import { PAGE_SIZES } from '@/shared/config';
import {
    boardPath,
    loginRedirectState,
    pageFromParam,
    topicPath,
    userPath,
} from '@/shared/lib';
import {
    ApiErrorState,
    Breadcrumbs,
    Loading,
    Pagination,
    RelativeTime,
} from '@/shared/ui';
import { PostList } from '@/widgets/post-list';

export function TopicPage() {
    const { t } = useTranslation();
    const { topicId = '' } = useParams();
    const [searchParams] = useSearchParams();
    const page = pageFromParam(searchParams.get('page'));
    const navigate = useNavigate();
    const location = useLocation();
    const { user, status } = useSession();
    const query = useTopicPage(topicId, page);
    const [insertion, setInsertion] = useState<ReplyInsertion>();

    // Scroll to a permalink (`#post-<id>`) once the posts are there.
    const hash = location.hash;
    const loaded = query.isSuccess;
    useEffect(() => {
        if (loaded && hash.startsWith('#post-')) {
            document.getElementById(hash.slice(1))?.scrollIntoView?.();
        }
    }, [loaded, hash]);

    if (query.isPending) return <Loading />;
    if (query.isError) {
        return (
            <ApiErrorState
                error={query.error}
                onRetry={() => void query.refetch()}
            />
        );
    }

    const { topic, posts } = query.data;
    // E.g. the only post of the last page was deleted: go to the new last page.
    if (posts.items.length === 0 && page > 1 && posts.totalPages > 0) {
        return (
            <Navigate
                to={topicPath(topic.id, topic.slug, { page: posts.totalPages })}
                replace
            />
        );
    }

    const postCount = topic.replyCount + 1;
    const mayReply = canReplyToTopic(user, topic);
    const onQuote = (text: string) =>
        setInsertion((previous) => ({ key: (previous?.key ?? 0) + 1, text }));
    const onPosted = (postId: string) => {
        // The new post is the last one: show the page it is on.
        const lastPage = pageOfPost(postCount + 1, PAGE_SIZES.posts);
        navigate(topicPath(topic.id, topic.slug, { page: lastPage, postId }));
    };

    return (
        <>
            <Breadcrumbs
                items={[
                    { label: t('forum.boardIndex'), to: '/forum' },
                    { label: topic.categoryName },
                    { label: topic.boardName, to: boardPath(topic.boardId) },
                    { label: topic.title },
                ]}
            />

            <div className="title-row">
                <div>
                    <h2 className="page-title">
                        <TopicBadges
                            pinned={topic.pinned}
                            locked={topic.locked}
                        />
                        {topic.title}
                    </h2>
                    <p>
                        {t('forum.startedBy')}{' '}
                        <Link to={userPath(topic.author.username)}>
                            {topic.author.displayName}
                        </Link>{' '}
                        &middot; <RelativeTime date={topic.createdAt} />{' '}
                        &middot; {t('forum.posts', { count: postCount })}
                    </p>
                </div>
                {(mayReply || status === 'guest') && (
                    <a className="btn" href="#reply">
                        {t('forum.postReply')}
                    </a>
                )}
            </div>

            {canModerateTopic(user) && (
                <ModerationBar topic={topic}>
                    <MoveTopicButton topic={topic} />
                </ModerationBar>
            )}

            <PostList topic={topic} posts={posts.items} onQuote={onQuote} />

            <Pagination current={posts.page} total={posts.totalPages} />

            <section
                className="reply"
                id="reply"
                aria-labelledby="reply-heading"
            >
                <h3 id="reply-heading">{t('forum.replyHeading')}</h3>
                {status === 'guest' ? (
                    <p className="reply-notice">
                        <Trans
                            i18nKey="forum.replyGuest"
                            components={{
                                login: (
                                    <Link
                                        to="/login"
                                        state={loginRedirectState(location)}
                                    />
                                ),
                                register: <Link to="/register" />,
                            }}
                        />
                    </p>
                ) : mayReply ? (
                    <ReplyForm
                        topicId={topic.id}
                        topicTitle={topic.title}
                        insertion={insertion}
                        onPosted={onPosted}
                    />
                ) : topic.locked ? (
                    <p className="reply-notice">{t('forum.replyLocked')}</p>
                ) : null}
            </section>
        </>
    );
}
