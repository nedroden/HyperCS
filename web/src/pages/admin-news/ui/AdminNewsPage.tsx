import { useTranslation } from 'react-i18next';
import { NewsManager } from '@/features/manage-news';
import { AdminLayout } from '@/widgets/admin-layout';

export function AdminNewsPage() {
    const { t } = useTranslation();
    return (
        <AdminLayout title={t('admin.news.title')}>
            <NewsManager />
        </AdminLayout>
    );
}
