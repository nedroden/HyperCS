import { Trans, useTranslation } from 'react-i18next';
import { Link } from 'react-router-dom';
import { CategorySection, useCategories } from '@/entities/category';
import { useSession } from '@/entities/session';
import { topicPath, userPath } from '@/shared/lib';
import { ApiErrorState, EmptyState, Loading, RelativeTime } from '@/shared/ui';
import { useStats, type Stats } from '@/entities/stat';

const TOTALS = ['topics', 'posts', 'members', 'boards'] as const;

function StatsSection({ stats }: { stats: Stats }) {
    const { t } = useTranslation();
    return (
        <section
            className="category stats-section"
            aria-labelledby="forum-stats"
        >
            <div className="category-head">
                <h3 id="forum-stats">{t('forum.stats.title')}</h3>
                <p>{t('forum.stats.intro')}</p>
            </div>
            <div className="stats-grid">
                {TOTALS.map((key) => (
                    <div className="stat" key={key}>
                        <div className="value">{stats[key]}</div>
                        <div className="label">{t(`forum.stats.${key}`)}</div>
                    </div>
                ))}
            </div>
            <div className="stats-extra">
                {stats.newestMember && (
                    <div>
                        {t('forum.stats.newestMember')}{' '}
                        <Link to={userPath(stats.newestMember.username)}>
                            {stats.newestMember.displayName}
                        </Link>
                    </div>
                )}
                {stats.latestPost && (
                    <div>
                        {t('forum.stats.latestPost')}{' '}
                        <Link to={topicPath(stats.latestPost.topicId)}>
                            {stats.latestPost.topicTitle}
                        </Link>{' '}
                        {t('forum.by')}{' '}
                        <Link to={userPath(stats.latestPost.author.username)}>
                            {stats.latestPost.author.displayName}
                        </Link>
                        , <RelativeTime date={stats.latestPost.createdAt} />
                    </div>
                )}
                <div>
                    {t('forum.stats.usersOnline', {
                        count: stats.onlineCount,
                    })}{' '}
                    {stats.onlineUsers.map((user) => (
                        <Link key={user.id} to={userPath(user.username)}>
                            {user.displayName}
                        </Link>
                    ))}
                </div>
            </div>
        </section>
    );
}

export function ForumIndexPage() {
    const { t } = useTranslation();
    const { status } = useSession();
    const categories = useCategories();
    const stats = useStats();

    return (
        <>
            <h2 className="page-title">{t('forum.boardIndex')}</h2>

            {status === 'guest' && (
                <p className="guest-banner">
                    <Trans
                        i18nKey="forum.guestBanner"
                        components={{
                            login: <Link to="/login" />,
                            register: <Link to="/register" />,
                        }}
                    />
                </p>
            )}

            {categories.isPending ? (
                <Loading />
            ) : categories.isError ? (
                <ApiErrorState
                    error={categories.error}
                    onRetry={() => void categories.refetch()}
                />
            ) : categories.data.categories.length === 0 ? (
                <EmptyState
                    title={t('forum.empty.categoriesTitle')}
                    text={t('forum.empty.categoriesText')}
                />
            ) : (
                categories.data.categories.map((category) => (
                    <CategorySection key={category.id} category={category} />
                ))
            )}

            {stats.data && <StatsSection stats={stats.data} />}
        </>
    );
}
