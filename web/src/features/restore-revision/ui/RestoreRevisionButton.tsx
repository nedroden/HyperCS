import { useTranslation } from 'react-i18next';
import { problemOf, queryKeys, useApiMutation } from '@/shared/api';
import { errorKey } from '@/shared/lib';
import { restoreRevision } from '../api/restoreRevision';

type RestoreRevisionButtonProps = {
    postId: string;
    revisionId: string;
    onRestored: () => void;
};

/** "Restore this version": a normal edit, so history is kept and a new revision is created. */
export function RestoreRevisionButton({
    postId,
    revisionId,
    onRestored,
}: RestoreRevisionButtonProps) {
    const { t } = useTranslation();
    const restore = useApiMutation({
        mutationFn: () => restoreRevision(postId, revisionId),
        invalidates: [queryKeys.topics.all, queryKeys.posts.all],
        onSuccess: onRestored,
    });
    return (
        <>
            <button
                className="btn"
                type="button"
                disabled={restore.isPending}
                onClick={() => restore.mutate()}
            >
                {t('forum.history.restore')}
            </button>
            {restore.error && (
                <p className="field-error" role="alert">
                    {t(errorKey(problemOf(restore.error).code), {
                        defaultValue: t('errors.unknown'),
                    })}
                </p>
            )}
        </>
    );
}
