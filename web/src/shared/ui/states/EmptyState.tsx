import type { ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

type EmptyStateProps = {
    title?: string;
    text?: string;
    /** Optional call to action, e.g. a "New topic" link. */
    action?: ReactNode;
};

/** Shown when a list or page has no content. */
export function EmptyState({ title, text, action }: EmptyStateProps) {
    const { t } = useTranslation();
    return (
        <div className="state state-empty">
            <p className="state-title">{title ?? t('state.empty.title')}</p>
            <p>{text ?? t('state.empty.text')}</p>
            {action}
        </div>
    );
}
