import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { problemOf, queryKeys, useApiMutation } from '@/shared/api';
import { boardPath, errorKey } from '@/shared/lib';
import { ConfirmDialog } from '@/shared/ui';
import { deletePost } from '../api/deletePost';

type DeletePostButtonProps = {
    postId: string;
    /** The first post: deleting it deletes the topic, so the dialog warns and we go back to the board. */
    isFirstPost: boolean;
    boardId: string;
};

/** "Delete" post action with a confirmation dialog. */
export function DeletePostButton({
    postId,
    isFirstPost,
    boardId,
}: DeletePostButtonProps) {
    const { t } = useTranslation();
    const navigate = useNavigate();
    const [open, setOpen] = useState(false);
    const remove = useApiMutation({
        mutationFn: () => deletePost(postId),
        invalidates: [
            queryKeys.topics.all,
            queryKeys.boards.all,
            queryKeys.categories.all,
            queryKeys.posts.all,
            queryKeys.stats.all,
        ],
    });

    const confirm = async () => {
        try {
            await remove.mutateAsync();
            setOpen(false);
            if (isFirstPost) navigate(boardPath(boardId));
        } catch {
            // The error is shown in the dialog.
        }
    };

    return (
        <>
            <button
                type="button"
                className="link-button"
                onClick={() => {
                    remove.reset();
                    setOpen(true);
                }}
            >
                {t('forum.delete')}
            </button>
            {open && (
                <ConfirmDialog
                    title={
                        isFirstPost
                            ? t('forum.deleteTopic.title')
                            : t('forum.deletePost.title')
                    }
                    text={
                        isFirstPost
                            ? t('forum.deletePost.firstPostWarning')
                            : t('forum.deletePost.text')
                    }
                    confirmLabel={t('forum.delete')}
                    cancelLabel={t('forum.form.cancel')}
                    pending={remove.isPending}
                    error={
                        remove.error
                            ? t(errorKey(problemOf(remove.error).code), {
                                  defaultValue: t('errors.unknown'),
                              })
                            : undefined
                    }
                    onConfirm={() => void confirm()}
                    onCancel={() => setOpen(false)}
                />
            )}
        </>
    );
}
