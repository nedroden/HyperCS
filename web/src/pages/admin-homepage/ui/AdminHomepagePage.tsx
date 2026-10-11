import { useTranslation } from 'react-i18next';
import { HomepageEditor } from '@/features/edit-homepage';
import { AdminLayout } from '@/widgets/admin-layout';

export function AdminHomepagePage() {
    const { t } = useTranslation();
    return (
        <AdminLayout title={t('admin.homepage.title')}>
            <HomepageEditor />
        </AdminLayout>
    );
}
