import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { problemOf } from '@/shared/api';
import { errorKey } from '@/shared/lib';
import {
    ConfirmDialog,
    EmptyState,
    ErrorState,
    Loading,
    translateError,
} from '@/shared/ui';
import type { Board, Category, Direction } from '../api/structure';
import { useStructure, useStructureMutations } from '../model/useStructure';
import { StructureForm } from './StructureForm';

type Dialog =
    | { kind: 'category-form'; category?: Category }
    | { kind: 'board-form'; category: Category; board?: Board }
    | { kind: 'delete-category'; category: Category }
    | { kind: 'delete-board'; board: Board };

type RowActionsProps = {
    first: boolean;
    last: boolean;
    labels: { up: string; down: string; edit: string; delete: string };
    onMove: (direction: Direction) => void;
    onEdit: () => void;
    onDelete: () => void;
};

function RowActions({
    first,
    last,
    labels,
    onMove,
    onEdit,
    onDelete,
}: RowActionsProps) {
    const { t } = useTranslation();
    return (
        <div className="row-actions">
            <button
                type="button"
                aria-label={labels.up}
                title={labels.up}
                disabled={first}
                onClick={() => onMove('up')}
            >
                <span aria-hidden="true">↑</span>
            </button>
            <button
                type="button"
                aria-label={labels.down}
                title={labels.down}
                disabled={last}
                onClick={() => onMove('down')}
            >
                <span aria-hidden="true">↓</span>
            </button>
            <button type="button" aria-label={labels.edit} onClick={onEdit}>
                {t('admin.actions.edit')}
            </button>
            <button type="button" aria-label={labels.delete} onClick={onDelete}>
                {t('admin.actions.delete')}
            </button>
        </div>
    );
}

/** Admin screen body: categories and boards with create / edit / delete / reorder (mocks/admin-structure.html). */
export function StructureManager() {
    const { t, i18n } = useTranslation();
    const structure = useStructure();
    const m = useStructureMutations();
    const [dialog, setDialog] = useState<Dialog | null>(null);
    const [actionError, setActionError] = useState<string | null>(null);
    const [deleteError, setDeleteError] = useState<string | null>(null);

    const errorText = (error: unknown) =>
        translateError(t, i18n, errorKey(problemOf(error).code));

    const close = () => {
        setDialog(null);
        setDeleteError(null);
    };

    const move = async (run: () => Promise<unknown>) => {
        setActionError(null);
        try {
            await run();
        } catch (error) {
            setActionError(errorText(error));
        }
    };

    const confirmDelete = async () => {
        if (!dialog) return;
        setDeleteError(null);
        try {
            if (dialog.kind === 'delete-category')
                await m.deleteCategory.mutateAsync(dialog.category.id);
            if (dialog.kind === 'delete-board')
                await m.deleteBoard.mutateAsync(dialog.board.id);
            close();
        } catch (error) {
            setDeleteError(errorText(error));
        }
    };

    if (structure.isPending) return <Loading />;
    if (structure.isError)
        return <ErrorState onRetry={() => void structure.refetch()} />;

    const categories = structure.data;
    const deleting = m.deleteCategory.isPending || m.deleteBoard.isPending;

    return (
        <>
            {actionError && (
                <p className="admin-alert" role="alert">
                    {t('admin.structure.actionFailed')} {actionError}
                </p>
            )}
            {categories.length === 0 && (
                <EmptyState text={t('admin.structure.empty')} />
            )}
            {categories.map((category, ci) => (
                <section
                    className="cat-block"
                    key={category.id}
                    aria-label={category.name}
                >
                    <div className="cat-head">
                        <div>
                            <h3 className="name">{category.name}</h3>
                            {category.adminOnly && (
                                <span className="flag-badge">
                                    {t('admin.structure.adminOnly')}
                                </span>
                            )}
                        </div>
                        <RowActions
                            first={ci === 0}
                            last={ci === categories.length - 1}
                            labels={{
                                up: t('admin.structure.moveCategoryUp', {
                                    name: category.name,
                                }),
                                down: t('admin.structure.moveCategoryDown', {
                                    name: category.name,
                                }),
                                edit: t('admin.structure.editNamed', {
                                    name: category.name,
                                }),
                                delete: t('admin.structure.deleteNamed', {
                                    name: category.name,
                                }),
                            }}
                            onMove={(direction) =>
                                void move(() =>
                                    m.reorderCategory.mutateAsync({
                                        id: category.id,
                                        direction,
                                    }),
                                )
                            }
                            onEdit={() =>
                                setDialog({ kind: 'category-form', category })
                            }
                            onDelete={() =>
                                setDialog({ kind: 'delete-category', category })
                            }
                        />
                    </div>
                    {category.boards.map((board, bi) => (
                        <div className="board-row" key={board.id}>
                            <span className="name">
                                {board.name}
                                {board.adminOnly && !category.adminOnly && (
                                    <span className="flag-badge">
                                        {t('admin.structure.adminOnly')}
                                    </span>
                                )}
                            </span>
                            <RowActions
                                first={bi === 0}
                                last={bi === category.boards.length - 1}
                                labels={{
                                    up: t('admin.structure.moveBoardUp', {
                                        name: board.name,
                                    }),
                                    down: t('admin.structure.moveBoardDown', {
                                        name: board.name,
                                    }),
                                    edit: t('admin.structure.editNamed', {
                                        name: board.name,
                                    }),
                                    delete: t('admin.structure.deleteNamed', {
                                        name: board.name,
                                    }),
                                }}
                                onMove={(direction) =>
                                    void move(() =>
                                        m.reorderBoard.mutateAsync({
                                            id: board.id,
                                            direction,
                                        }),
                                    )
                                }
                                onEdit={() =>
                                    setDialog({
                                        kind: 'board-form',
                                        category,
                                        board,
                                    })
                                }
                                onDelete={() =>
                                    setDialog({ kind: 'delete-board', board })
                                }
                            />
                        </div>
                    ))}
                    <div className="add-row">
                        <button
                            className="btn"
                            type="button"
                            aria-label={t('admin.structure.addBoardTo', {
                                category: category.name,
                            })}
                            onClick={() =>
                                setDialog({ kind: 'board-form', category })
                            }
                        >
                            {t('admin.structure.addBoard')}
                        </button>
                    </div>
                </section>
            ))}
            <button
                className="btn"
                type="button"
                onClick={() => setDialog({ kind: 'category-form' })}
            >
                {t('admin.structure.addCategory')}
            </button>

            {dialog?.kind === 'category-form' && (
                <StructureForm
                    title={
                        dialog.category
                            ? t('admin.structure.editCategory')
                            : t('admin.structure.newCategory')
                    }
                    initial={dialog.category}
                    onCancel={close}
                    onSubmit={async (values) => {
                        if (dialog.category)
                            await m.updateCategory.mutateAsync({
                                id: dialog.category.id,
                                body: values,
                            });
                        else await m.createCategory.mutateAsync(values);
                        close();
                    }}
                />
            )}
            {dialog?.kind === 'board-form' && (
                <StructureForm
                    title={
                        dialog.board
                            ? t('admin.structure.editBoard')
                            : t('admin.structure.newBoard')
                    }
                    initial={
                        dialog.board && {
                            ...dialog.board,
                            // The category's flag may be reflected in the board's; only the board's own counts here.
                            adminOnly:
                                dialog.board.adminOnly &&
                                !dialog.category.adminOnly,
                        }
                    }
                    onCancel={close}
                    onSubmit={async (values) => {
                        if (dialog.board)
                            await m.updateBoard.mutateAsync({
                                id: dialog.board.id,
                                body: values,
                            });
                        else
                            await m.createBoard.mutateAsync({
                                ...values,
                                categoryId: dialog.category.id,
                            });
                        close();
                    }}
                />
            )}
            {dialog?.kind === 'delete-category' && (
                <ConfirmDialog
                    title={t('admin.structure.deleteCategoryTitle', {
                        name: dialog.category.name,
                    })}
                    text={t('admin.structure.deleteCategoryText', {
                        count: dialog.category.boards.length,
                    })}
                    confirmLabel={t('admin.structure.confirmDelete')}
                    cancelLabel={t('admin.actions.cancel')}
                    pending={deleting}
                    error={deleteError ?? undefined}
                    onConfirm={() => void confirmDelete()}
                    onCancel={close}
                />
            )}
            {dialog?.kind === 'delete-board' && (
                <ConfirmDialog
                    title={t('admin.structure.deleteBoardTitle', {
                        name: dialog.board.name,
                    })}
                    text={t('admin.structure.deleteBoardText')}
                    confirmLabel={t('admin.structure.confirmDelete')}
                    cancelLabel={t('admin.actions.cancel')}
                    pending={deleting}
                    error={deleteError ?? undefined}
                    onConfirm={() => void confirmDelete()}
                    onCancel={close}
                />
            )}
        </>
    );
}
