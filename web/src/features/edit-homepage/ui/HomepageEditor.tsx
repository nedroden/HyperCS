import { useTranslation } from 'react-i18next';
import { useHomepage, useTerms } from '@/entities/cms-page';
import { ErrorState, Loading } from '@/shared/ui';
import { FeaturesForm } from './FeaturesForm';
import { HeroForm } from './HeroForm';
import { TermsForm } from './TermsForm';

/** The editable CMS content: hero, feature blocks and terms of service. */
export function HomepageEditor() {
    const { t } = useTranslation();
    const homepage = useHomepage();
    const terms = useTerms();
    if (homepage.isPending || terms.isPending) return <Loading />;
    if (homepage.isError || terms.isError)
        return (
            <ErrorState
                text={t('admin.homepage.loadError')}
                onRetry={() => {
                    void homepage.refetch();
                    void terms.refetch();
                }}
            />
        );
    return (
        <>
            <HeroForm content={homepage.data} />
            <FeaturesForm content={homepage.data} />
            <TermsForm content={terms.data} />
        </>
    );
}
