import { useTranslation } from 'react-i18next';
import { useNavigate, useParams, useSearchParams } from 'react-router-dom';
import {
    canRestoreRevision,
    canViewPostHistory,
    PostForbidden,
    usePost,
    useRevisions,
} from '@/entities/post';
import { useSession } from '@/entities/session';
import { RestoreRevisionButton } from '@/features/restore-revision';
import { boardPath, pageFromParam, topicPath } from '@/shared/lib';
import {
    ApiErrorState,
    Breadcrumbs,
    EmptyState,
    Loading,
    Markdown,
    Pagination,
    RelativeTime,
} from '@/shared/ui';

/** `/forum/posts/:postId/history` (logged-in users only): earlier versions of a post, newest first. */
export function PostHistoryPage() {
    const { t } = useTranslation();
    const navigate = useNavigate();
    const [searchParams] = useSearchParams();
    const page = pageFromParam(searchParams.get('page'));
    const { user, status } = useSession();
    const { postId } = useParams();
    const query = usePost(postId);
    const post = query.data?.post;
    const allowed = !!post && canViewPostHistory(user, post);
    const revisions = useRevisions(post?.id ?? '', page, { enabled: allowed });

    if (query.isError) {
        return (
            <ApiErrorState
                error={query.error}
                onRetry={() => void query.refetch()}
            />
        );
    }
    if (!query.data || !post || status === 'loading') return <Loading />;
    if (!allowed) return <PostForbidden />;

    const { context } = query.data;
    const mayRestore = canRestoreRevision(user, post);
    const toTopic = () =>
        navigate(
            topicPath(context.topicId, context.topicSlug, {
                page: context.page,
                postId: post.id,
            }),
        );

    return (
        <>
            <Breadcrumbs
                items={[
                    { label: t('forum.boardIndex'), to: '/forum' },
                    { label: context.categoryName },
                    {
                        label: context.boardName,
                        to: boardPath(context.boardId),
                    },
                    {
                        label: context.topicTitle,
                        to: topicPath(context.topicId, context.topicSlug),
                    },
                    { label: t('forum.historyTitle') },
                ]}
            />
            <h2 className="page-title">{t('forum.historyTitle')}</h2>

            {page === 1 && (
                <section className="revision current">
                    <div className="rev-head">
                        <span>
                            {t('forum.history.current')} &middot;{' '}
                            {post.editedBy
                                ? t('forum.history.editedBy', {
                                      name: post.editedBy.displayName,
                                  })
                                : t('forum.history.postedBy', {
                                      name: post.author.displayName,
                                  })}{' '}
                            &middot;{' '}
                            <RelativeTime
                                date={post.editedAt ?? post.createdAt}
                            />
                        </span>
                    </div>
                    <div className="rev-body">
                        <Markdown>{post.body}</Markdown>
                    </div>
                </section>
            )}

            {revisions.isPending ? (
                <Loading />
            ) : revisions.isError ? (
                <ApiErrorState
                    error={revisions.error}
                    onRetry={() => void revisions.refetch()}
                />
            ) : revisions.data.items.length === 0 ? (
                <EmptyState
                    title={t('forum.history.emptyTitle')}
                    text={t('forum.history.emptyText')}
                />
            ) : (
                <>
                    {revisions.data.items.map((revision) => (
                        <section className="revision" key={revision.id}>
                            <div className="rev-head">
                                <span>
                                    {t('forum.history.replaced')}{' '}
                                    <RelativeTime date={revision.createdAt} />
                                    {revision.editedBy &&
                                        ` · ${t('forum.history.editedBy', {
                                            name: revision.editedBy.displayName,
                                        })}`}
                                </span>
                            </div>
                            <div className="rev-body">
                                <Markdown>{revision.body}</Markdown>
                            </div>
                            {mayRestore && (
                                <div className="rev-actions">
                                    <RestoreRevisionButton
                                        postId={post.id}
                                        revisionId={revision.id}
                                        onRestored={toTopic}
                                    />
                                </div>
                            )}
                        </section>
                    ))}
                    <Pagination
                        current={revisions.data.page}
                        total={revisions.data.totalPages}
                    />
                </>
            )}
        </>
    );
}
