import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSearchParams } from 'react-router-dom';
import { useNewsList, type NewsItem } from '@/entities/news-item';
import { problemOf } from '@/shared/api';
import { errorKey } from '@/shared/lib';
import {
    ConfirmDialog,
    EmptyState,
    ErrorState,
    Loading,
    Pagination,
    RelativeTime,
    translateError,
} from '@/shared/ui';
import { useNewsMutations } from '../model/useNewsMutations';
import { NewsForm } from './NewsForm';

type Dialog =
    { kind: 'form'; item?: NewsItem } | { kind: 'delete'; item: NewsItem };

function currentPage(params: URLSearchParams): number {
    const page = Number(params.get('page'));
    return Number.isInteger(page) && page >= 1 ? page : 1;
}

/** News list for admins with add / edit / delete (Markdown body with preview in the form). */
export function NewsManager() {
    const { t, i18n } = useTranslation();
    const [params] = useSearchParams();
    const page = currentPage(params);
    const news = useNewsList({ page });
    const m = useNewsMutations();
    const [dialog, setDialog] = useState<Dialog | null>(null);
    const [deleteError, setDeleteError] = useState<string | null>(null);

    const close = () => {
        setDialog(null);
        setDeleteError(null);
    };

    const confirmDelete = async (item: NewsItem) => {
        setDeleteError(null);
        try {
            await m.remove.mutateAsync(item.id);
            close();
        } catch (error) {
            setDeleteError(
                translateError(t, i18n, errorKey(problemOf(error).code)),
            );
        }
    };

    if (news.isPending) return <Loading />;
    if (news.isError)
        return (
            <ErrorState
                text={t('admin.news.loadError')}
                onRetry={() => void news.refetch()}
            />
        );
    const { items, totalPages } = news.data;

    return (
        <>
            <div className="admin-toolbar">
                <button
                    type="button"
                    className="btn"
                    onClick={() => setDialog({ kind: 'form' })}
                >
                    {t('admin.news.add')}
                </button>
            </div>
            {items.length === 0 ? (
                <EmptyState text={t('admin.news.empty')} />
            ) : (
                <ul className="news-admin-list">
                    {items.map((item) => (
                        <li className="news-row" key={item.id}>
                            <span>
                                <span className="news-row-title">
                                    {item.title}
                                </span>
                                <span className="news-row-date">
                                    <RelativeTime date={item.publishedAt} />
                                </span>
                            </span>
                            <span className="row-actions">
                                <button
                                    type="button"
                                    aria-label={t('admin.news.editNamed', {
                                        name: item.title,
                                    })}
                                    onClick={() =>
                                        setDialog({ kind: 'form', item })
                                    }
                                >
                                    {t('admin.actions.edit')}
                                </button>
                                <button
                                    type="button"
                                    aria-label={t('admin.news.deleteNamed', {
                                        name: item.title,
                                    })}
                                    onClick={() =>
                                        setDialog({ kind: 'delete', item })
                                    }
                                >
                                    {t('admin.actions.delete')}
                                </button>
                            </span>
                        </li>
                    ))}
                </ul>
            )}
            {totalPages > 1 && <Pagination current={page} total={totalPages} />}

            {dialog?.kind === 'form' && (
                <NewsForm
                    title={
                        dialog.item
                            ? t('admin.news.editTitle')
                            : t('admin.news.newTitle')
                    }
                    initial={
                        dialog.item && {
                            title: dialog.item.title,
                            body: dialog.item.body,
                        }
                    }
                    onCancel={close}
                    onSubmit={async (values) => {
                        if (dialog.item)
                            await m.update.mutateAsync({
                                id: dialog.item.id,
                                body: values,
                            });
                        else await m.create.mutateAsync(values);
                        close();
                    }}
                />
            )}
            {dialog?.kind === 'delete' && (
                <ConfirmDialog
                    title={t('admin.news.deleteTitle', {
                        name: dialog.item.title,
                    })}
                    text={t('admin.news.deleteText')}
                    confirmLabel={t('admin.actions.delete')}
                    cancelLabel={t('admin.actions.cancel')}
                    pending={m.remove.isPending}
                    error={deleteError ?? undefined}
                    onConfirm={() => void confirmDelete(dialog.item)}
                    onCancel={close}
                />
            )}
        </>
    );
}
