import { useTranslation } from 'react-i18next';
import { SiteSettingsForm } from './SiteSettingsForm';
import { AdminLayout } from '@/widgets/admin-layout';

export function AdminSettingsPage() {
    const { t } = useTranslation();
    return (
        <AdminLayout title={t('admin.settings.pageTitle')}>
            <SiteSettingsForm />
        </AdminLayout>
    );
}
