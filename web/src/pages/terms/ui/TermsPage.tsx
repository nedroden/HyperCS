import { useTranslation } from 'react-i18next';
import { useTerms } from '@/entities/cms-page';
import { ErrorState, Loading, Markdown } from '@/shared/ui';

/** Terms of service, managed in the admin area. The text is Markdown (plain text is valid Markdown). */
export function TermsPage() {
    const { t } = useTranslation();
    const terms = useTerms();
    return (
        <>
            <h2 className="page-title">{t('terms.title')}</h2>
            {terms.isPending ? (
                <Loading />
            ) : terms.isError ? (
                <ErrorState
                    text={t('terms.loadError')}
                    onRetry={() => void terms.refetch()}
                />
            ) : (
                <div className="cms-text">
                    <Markdown>{terms.data.body}</Markdown>
                </div>
            )}
        </>
    );
}
