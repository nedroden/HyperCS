import { useTranslation } from 'react-i18next';
import { Link, Navigate, useParams, useSearchParams } from 'react-router-dom';
import { useBoardPage } from '@/entities/board';
import { useSession } from '@/entities/session';
import { boardPath, newTopicPath, pageFromParam } from '@/shared/lib';
import {
    ApiErrorState,
    Breadcrumbs,
    EmptyState,
    Loading,
    Pagination,
} from '@/shared/ui';
import { TopicList } from '@/widgets/topic-list';

export function BoardPage() {
    const { t } = useTranslation();
    const { boardId = '' } = useParams();
    const [searchParams] = useSearchParams();
    const page = pageFromParam(searchParams.get('page'));
    const { status } = useSession();
    const query = useBoardPage(boardId, page);

    if (query.isPending) return <Loading />;
    if (query.isError) {
        return (
            <ApiErrorState
                error={query.error}
                onRetry={() => void query.refetch()}
            />
        );
    }

    const { board, topics } = query.data;
    // E.g. the last topic of the last page was deleted: go to the new last page.
    if (topics.items.length === 0 && page > 1 && topics.totalPages > 0) {
        return <Navigate to={boardPath(board.id, topics.totalPages)} replace />;
    }
    const canCreate = status === 'authenticated';
    const newTopicLink = (
        <Link className="btn" to={newTopicPath(board.id)}>
            {t('forum.newTopic')}
        </Link>
    );

    return (
        <>
            <Breadcrumbs
                items={[
                    { label: t('forum.boardIndex'), to: '/forum' },
                    { label: board.categoryName },
                    { label: board.name },
                ]}
            />

            <div className="title-row">
                <div>
                    <h2 className="page-title">{board.name}</h2>
                    {board.description && <p>{board.description}</p>}
                </div>
                {canCreate && newTopicLink}
            </div>

            {topics.items.length === 0 ? (
                <EmptyState
                    title={t('forum.empty.topicsTitle')}
                    text={t('forum.empty.topicsText')}
                    action={canCreate ? newTopicLink : undefined}
                />
            ) : (
                <>
                    <TopicList topics={topics.items} />
                    <Pagination
                        current={topics.page}
                        total={topics.totalPages}
                    />
                </>
            )}
        </>
    );
}
