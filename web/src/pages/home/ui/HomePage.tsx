import { useTranslation } from 'react-i18next';
import { ApiStatus } from '@/entities/api-health';
import { useRelativeTime } from '@/shared/lib';
import { features, news } from '../model/placeholder';

export function HomePage() {
    const { t } = useTranslation();
    const ago = useRelativeTime();
    return (
        <>
            <section className="hero">
                <h2>{t('home.title')}</h2>
                <p>{t('home.intro')}</p>
                <a className="btn" href="#">
                    {t('home.learnMore')}
                </a>
            </section>

            <section className="features">
                {features.map((f) => (
                    <div className="feature" key={f.title}>
                        <h3>{f.title}</h3>
                        <p>{f.text}</p>
                    </div>
                ))}
            </section>

            <div className="section-head">
                <h3>{t('home.latestNews')}</h3>
            </div>
            <section className="news">
                {news.map((n) => (
                    <article key={n.title}>
                        <h4>
                            <a href="#">{n.title}</a>
                        </h4>
                        <div className="date">{ago(n.days)}</div>
                        <p>{n.text}</p>
                    </article>
                ))}
            </section>

            <ApiStatus />
        </>
    );
}
