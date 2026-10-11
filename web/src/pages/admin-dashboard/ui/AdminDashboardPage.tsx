import { useTranslation } from 'react-i18next';
import { Link } from 'react-router-dom';
import { useStats } from '@/entities/stat';
import { ApiErrorState, Icon, Loading } from '@/shared/ui';
import { ADMIN_SECTIONS, AdminLayout } from '@/widgets/admin-layout';

const TOTALS = ['members', 'topics', 'posts', 'boards', 'onlineCount'] as const;

/** Admin start page: community statistics and large links to every admin section. */
export function AdminDashboardPage() {
    const { t } = useTranslation();
    const stats = useStats();
    return (
        <AdminLayout title={t('admin.dashboard.title')}>
            <section aria-labelledby="admin-stats">
                <h3 className="admin-section-title" id="admin-stats">
                    {t('admin.dashboard.statistics')}
                </h3>
                {stats.isPending ? (
                    <Loading />
                ) : stats.isError ? (
                    <ApiErrorState
                        error={stats.error}
                        onRetry={() => void stats.refetch()}
                    />
                ) : (
                    <div className="stats-grid admin-stats">
                        {TOTALS.map((key) => (
                            <div className="stat" key={key}>
                                <div className="value">{stats.data[key]}</div>
                                <div className="label">
                                    {t(`admin.dashboard.${key}`)}
                                </div>
                            </div>
                        ))}
                    </div>
                )}
            </section>

            <section aria-labelledby="admin-sections">
                <h3 className="admin-section-title" id="admin-sections">
                    {t('admin.dashboard.sections')}
                </h3>
                <div className="admin-tiles">
                    {ADMIN_SECTIONS.map((s) => (
                        <Link className="admin-tile" key={s.key} to={s.to}>
                            <Icon name={s.icon} size={48} />
                            <span className="admin-tile-title">
                                {t(`admin.nav.${s.key}`)}
                            </span>
                            <span className="admin-tile-text">
                                {t(`admin.dashboard.${s.key}Text`)}
                            </span>
                        </Link>
                    ))}
                </div>
            </section>
        </AdminLayout>
    );
}
