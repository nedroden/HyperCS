import { Suspense } from 'react';
import { useTranslation } from 'react-i18next';
import { Outlet } from 'react-router-dom';
import { useSiteSettingsEffects } from '@/app/lib/useSiteSettingsEffects';
import { Footer } from '@/widgets/footer';
import { Loading } from '@/shared/ui';
import { Header } from '@/widgets/header';

export function MainLayout() {
    const { t } = useTranslation();
    useSiteSettingsEffects();
    return (
        <div className="wrapper">
            <a className="skip-link" href="#main-content">
                {t('a11y.skipToContent')}
            </a>
            <Header />
            <main className="site-main" id="main-content" tabIndex={-1}>
                <Suspense fallback={<Loading />}>
                    <Outlet />
                </Suspense>
            </main>
            <Footer />
        </div>
    );
}
