import { useTranslation } from 'react-i18next';
import { Link, useNavigate, useParams } from 'react-router-dom';
import {
    canEditPost,
    canViewPostHistory,
    PostForbidden,
    usePost,
    useRevisions,
} from '@/entities/post';
import { useSession } from '@/entities/session';
import { EditPostForm } from '@/features/edit-post';
import { boardPath, postHistoryPath, topicPath } from '@/shared/lib';
import { ApiErrorState, Breadcrumbs, Loading } from '@/shared/ui';

/** `/forum/posts/:postId/edit` (logged-in users only). Loads the post by id, so a direct link or reload works. */
export function EditPostPage() {
    const { t } = useTranslation();
    const navigate = useNavigate();
    const { user, status } = useSession();
    const { postId } = useParams();
    const query = usePost(postId);
    const post = query.data?.post;
    const showHistory =
        !!post && !!post.editedAt && canViewPostHistory(user, post);
    const revisions = useRevisions(post?.id ?? '', 1, {
        enabled: showHistory,
    });

    if (query.isError) {
        return (
            <ApiErrorState
                error={query.error}
                onRetry={() => void query.refetch()}
            />
        );
    }
    if (!query.data || status === 'loading') return <Loading />;
    if (!canEditPost(user, query.data.post)) return <PostForbidden />;

    const { post: loaded, context } = query.data;
    const back = () =>
        navigate(
            topicPath(context.topicId, context.topicSlug, {
                page: context.page,
                postId: loaded.id,
            }),
        );
    const revisionCount = revisions.data?.total;

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
                    { label: t('forum.editPostTitle') },
                ]}
            />
            <h2 className="page-title">{t('forum.editPostTitle')}</h2>
            <EditPostForm
                postId={loaded.id}
                initialBody={loaded.body}
                onSaved={back}
                onCancel={back}
                extra={
                    showHistory && (
                        <Link
                            className="history-link"
                            to={postHistoryPath(loaded.id)}
                        >
                            {revisionCount === undefined
                                ? t('forum.history.link')
                                : t('forum.history.linkCount', {
                                      count: revisionCount,
                                  })}
                        </Link>
                    )
                }
            />
        </>
    );
}
