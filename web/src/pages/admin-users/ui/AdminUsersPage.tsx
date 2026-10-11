import { useTranslation } from 'react-i18next';
import { UsersTable } from '@/features/manage-users';
import { AdminLayout } from '@/widgets/admin-layout';

export function AdminUsersPage() {
    const { t } = useTranslation();
    return (
        <AdminLayout title={t('admin.users.title')}>
            <UsersTable />
        </AdminLayout>
    );
}
