import { useTranslation } from 'react-i18next';
import { Link } from 'react-router-dom';
import { problemOf } from '@/shared/api';
import { ErrorState } from './ErrorState';

type ApiErrorStateProps = {
    /** The error thrown by a query (an `ApiError`, or anything else). */
    error: unknown;
    onRetry?: () => void;
};

/**
 * Error state for a failed data request: 404 -> not found, 401/403 -> no access, anything else -> the
 * generic message with a retry button.
 */
export function ApiErrorState({ error, onRetry }: ApiErrorStateProps) {
    const { t } = useTranslation();
    const status = problemOf(error).status;
    if (status === 404) {
        return (
            <ErrorState
                title={t('apiState.notFound.title')}
                text={t('apiState.notFound.text')}
                action={<Link to="/">{t('common.backHome')}</Link>}
            />
        );
    }
    if (status === 401 || status === 403) {
        return (
            <ErrorState
                title={t('apiState.forbidden.title')}
                text={t('apiState.forbidden.text')}
                action={<Link to="/">{t('common.backHome')}</Link>}
            />
        );
    }
    return <ErrorState onRetry={onRetry} />;
}
