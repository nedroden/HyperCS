import { useTranslation } from 'react-i18next';
import { Link } from 'react-router-dom';
import { useHomepage } from '@/entities/cms-page';
import { NewsItemCard, useNewsList } from '@/entities/news-item';
import { useSession } from '@/entities/session';
import { useSiteSettings } from '@/entities/site-setting';
import { EmptyState, ErrorState, Loading } from '@/shared/ui';

const LATEST_NEWS_COUNT = 5;

function LatestNews() {
    const { t } = useTranslation();
    const news = useNewsList({ perPage: LATEST_NEWS_COUNT });
    return (
        <>
            <div className="section-head">
                <h3>{t('home.latestNews')}</h3>
            </div>
            {news.isPending ? (
                <Loading />
            ) : news.isError ? (
                <ErrorState
                    text={t('home.newsLoadError')}
                    onRetry={() => void news.refetch()}
                />
            ) : news.data.items.length === 0 ? (
                <EmptyState text={t('home.noNews')} />
            ) : (
                <section className="news">
                    {news.data.items.map((item) => (
                        <NewsItemCard key={item.id} item={item} />
                    ))}
                </section>
            )}
        </>
    );
}

/** Public homepage: hero, three feature blocks and the latest news, all loaded from the CMS API. */
export function HomePage() {
    const { t } = useTranslation();
    const { status } = useSession();
    const homepage = useHomepage();
    const slogan = useSiteSettings().data?.slogan;

    return (
        <>
            {homepage.isPending ? (
                <Loading />
            ) : homepage.isError ? (
                <ErrorState
                    text={t('home.loadError')}
                    onRetry={() => void homepage.refetch()}
                />
            ) : (
                <>
                    <section className="hero">
                        <h2>{homepage.data.heroHeading}</h2>
                        {slogan && <p>{slogan}</p>}
                        {status === 'guest' && (
                            <Link className="btn" to="/register">
                                {homepage.data.heroButtonLabel}
                            </Link>
                        )}
                    </section>

                    <section className="features">
                        {homepage.data.features.map((feature, index) => (
                            <div className="feature" key={index}>
                                <h3>{feature.title}</h3>
                                <p>{feature.text}</p>
                            </div>
                        ))}
                    </section>
                </>
            )}

            <LatestNews />
        </>
    );
}
