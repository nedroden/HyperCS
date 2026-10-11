import { useState, type ReactNode } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { problemOf, queryKeys, useApiMutation } from '@/shared/api';
import { boardPath, errorKey } from '@/shared/lib';
import { ConfirmDialog } from '@/shared/ui';
import { deleteTopic, setTopicFlags } from '../api/topicModeration';

type ModerationBarProps = {
    topic: { id: string; boardId: string; pinned: boolean; locked: boolean };
    /** More moderation controls, e.g. the move-topic button. */
    children?: ReactNode;
};

const INVALIDATES = [
    queryKeys.topics.all,
    queryKeys.boards.all,
    queryKeys.categories.all,
    queryKeys.stats.all,
];

/** Pin/unpin, lock/unlock and delete for moderators and admins. Render it only for those roles. */
export function ModerationBar({ topic, children }: ModerationBarProps) {
    const { t } = useTranslation();
    const navigate = useNavigate();
    const [confirmDelete, setConfirmDelete] = useState(false);
    const flags = useApiMutation({
        mutationFn: (value: { pinned?: boolean; locked?: boolean }) =>
            setTopicFlags(topic.id, value),
        invalidates: INVALIDATES,
    });
    const remove = useApiMutation({
        mutationFn: () => deleteTopic(topic.id),
        invalidates: INVALIDATES,
    });

    const onDelete = async () => {
        try {
            await remove.mutateAsync();
            setConfirmDelete(false);
            navigate(boardPath(topic.boardId));
        } catch {
            // Shown in the dialog.
        }
    };
    const errorText = (error: unknown) =>
        t(errorKey(problemOf(error).code), {
            defaultValue: t('errors.unknown'),
        });

    return (
        <div
            className="mod-bar"
            role="group"
            aria-label={t('forum.moderation.label')}
        >
            <button
                type="button"
                className="btn secondary"
                disabled={flags.isPending}
                onClick={() => flags.mutate({ pinned: !topic.pinned })}
            >
                {topic.pinned
                    ? t('forum.moderation.unpin')
                    : t('forum.moderation.pin')}
            </button>
            <button
                type="button"
                className="btn secondary"
                disabled={flags.isPending}
                onClick={() => flags.mutate({ locked: !topic.locked })}
            >
                {topic.locked
                    ? t('forum.moderation.unlock')
                    : t('forum.moderation.lock')}
            </button>
            {children}
            <button
                type="button"
                className="btn secondary"
                onClick={() => {
                    remove.reset();
                    setConfirmDelete(true);
                }}
            >
                {t('forum.moderation.deleteTopic')}
            </button>
            {flags.error && (
                <p className="field-error" role="alert">
                    {errorText(flags.error)}
                </p>
            )}
            {confirmDelete && (
                <ConfirmDialog
                    title={t('forum.deleteTopic.title')}
                    text={t('forum.deleteTopic.text')}
                    confirmLabel={t('forum.delete')}
                    cancelLabel={t('forum.form.cancel')}
                    pending={remove.isPending}
                    error={remove.error ? errorText(remove.error) : undefined}
                    onConfirm={() => void onDelete()}
                    onCancel={() => setConfirmDelete(false)}
                />
            )}
        </div>
    );
}
