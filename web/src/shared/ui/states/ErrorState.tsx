import type { ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

type ErrorStateProps = {
    title?: string;
    text?: string;
    /** Shows a "Try again" button that calls this. */
    onRetry?: () => void;
    /** Replaces the retry button, e.g. a reload button or a link home. */
    action?: ReactNode;
};

/** Shown when loading data failed (`role="alert"`). */
export function ErrorState({ title, text, onRetry, action }: ErrorStateProps) {
    const { t } = useTranslation();
    return (
        <div className="state state-error" role="alert">
            <p className="state-title">{title ?? t('state.error.title')}</p>
            <p>{text ?? t('state.error.text')}</p>
            {action ??
                (onRetry && (
                    <button type="button" className="btn" onClick={onRetry}>
                        {t('common.retry')}
                    </button>
                ))}
        </div>
    );
}
