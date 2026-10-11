import { useTranslation } from 'react-i18next';
import { useParams } from 'react-router-dom';
import { useBoardPage } from '@/entities/board';
import { CreateTopicForm } from '@/features/create-topic';
import { boardPath } from '@/shared/lib';
import { ApiErrorState, Breadcrumbs, Loading } from '@/shared/ui';

/** `/forum/boards/:boardId/new-topic` (logged-in users only, see `RequireAuth` in `app/App.tsx`). */
export function NewTopicPage() {
    const { t } = useTranslation();
    const { boardId = '' } = useParams();
    // Loads the board for the breadcrumbs and to surface "not found" / "no access" early.
    const query = useBoardPage(boardId, 1);

    if (query.isPending) return <Loading />;
    if (query.isError) {
        return (
            <ApiErrorState
                error={query.error}
                onRetry={() => void query.refetch()}
            />
        );
    }
    const { board } = query.data;

    return (
        <>
            <Breadcrumbs
                items={[
                    { label: t('forum.boardIndex'), to: '/forum' },
                    { label: board.categoryName },
                    { label: board.name, to: boardPath(board.id) },
                    { label: t('forum.newTopic') },
                ]}
            />
            <h2 className="page-title">{t('forum.newTopic')}</h2>
            <CreateTopicForm boardId={board.id} />
        </>
    );
}
