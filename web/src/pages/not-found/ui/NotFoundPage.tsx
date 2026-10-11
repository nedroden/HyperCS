import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Link } from 'react-router-dom';
import { useSiteSettings } from '@/entities/site-setting';

export function NotFoundPage() {
    const { t } = useTranslation();
    const siteTitle = useSiteSettings().data?.siteTitle ?? t('app.name');
    useEffect(() => {
        const previous = document.title;
        document.title = `${t('notFound.documentTitle')} – ${siteTitle}`;
        return () => {
            document.title = previous;
        };
    }, [t, siteTitle]);
    return (
        <section className="not-found">
            <p className="not-found-code" aria-hidden="true">
                {t('notFound.code')}
            </p>
            <h2>{t('notFound.title')}</h2>
            <Link className="btn" to="/">
                {t('notFound.back')}
            </Link>
        </section>
    );
}
