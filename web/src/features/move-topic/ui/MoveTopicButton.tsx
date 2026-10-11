import { useId, useState, type FormEvent } from 'react';
import { useTranslation } from 'react-i18next';
import { useCategories } from '@/entities/category';
import { problemOf, queryKeys, useApiMutation } from '@/shared/api';
import { errorKey } from '@/shared/lib';
import { Dialog, Loading } from '@/shared/ui';
import { moveTopic } from '../api/moveTopic';

type MoveTopicButtonProps = {
    topic: { id: string; title: string; boardId: string };
};

/** "Move topic" for moderators and admins: a dialog with a destination-board dropdown (mocks/move-topic.html). */
export function MoveTopicButton({ topic }: MoveTopicButtonProps) {
    const { t } = useTranslation();
    const [open, setOpen] = useState(false);
    return (
        <>
            <button
                type="button"
                className="btn secondary"
                onClick={() => setOpen(true)}
            >
                {t('forum.moderation.move')}
            </button>
            {open && (
                <MoveDialog topic={topic} onClose={() => setOpen(false)} />
            )}
        </>
    );
}

function MoveDialog({
    topic,
    onClose,
}: MoveTopicButtonProps & { onClose: () => void }) {
    const { t } = useTranslation();
    const selectId = useId();
    const categories = useCategories();
    const [target, setTarget] = useState('');
    const move = useApiMutation({
        mutationFn: (boardId: string) => moveTopic(topic.id, boardId),
        invalidates: [
            queryKeys.topics.all,
            queryKeys.boards.all,
            queryKeys.categories.all,
        ],
    });

    const groups = (categories.data?.categories ?? [])
        .map((category) => ({
            ...category,
            boards: category.boards.filter((b) => b.id !== topic.boardId),
        }))
        .filter((category) => category.boards.length > 0);
    const firstBoard = groups[0]?.boards[0]?.id ?? '';
    const selected = target || firstBoard;

    const onSubmit = async (event: FormEvent) => {
        event.preventDefault();
        if (!selected) return;
        try {
            await move.mutateAsync(selected);
            onClose();
        } catch {
            // Shown below.
        }
    };

    return (
        <Dialog
            title={t('forum.move.title', { title: topic.title })}
            onClose={onClose}
        >
            {categories.isPending ? (
                <Loading />
            ) : categories.isError ? (
                <p className="field-error" role="alert">
                    {t('state.error.text')}
                </p>
            ) : (
                <form onSubmit={(e) => void onSubmit(e)}>
                    <label htmlFor={selectId}>
                        {t('forum.move.destination')}
                    </label>
                    <select
                        id={selectId}
                        value={selected}
                        onChange={(e) => setTarget(e.target.value)}
                    >
                        {groups.map((category) => (
                            <optgroup key={category.id} label={category.name}>
                                {category.boards.map((board) => (
                                    <option key={board.id} value={board.id}>
                                        {board.name}
                                    </option>
                                ))}
                            </optgroup>
                        ))}
                    </select>
                    {move.error && (
                        <p className="field-error" role="alert">
                            {t(errorKey(problemOf(move.error).code), {
                                defaultValue: t('errors.unknown'),
                            })}
                        </p>
                    )}
                    <div className="dialog-actions">
                        <button
                            type="submit"
                            className="btn"
                            disabled={!selected || move.isPending}
                        >
                            {t('forum.move.submit')}
                        </button>
                        <button
                            type="button"
                            className="btn secondary"
                            onClick={onClose}
                            disabled={move.isPending}
                        >
                            {t('forum.form.cancel')}
                        </button>
                    </div>
                </form>
            )}
        </Dialog>
    );
}
