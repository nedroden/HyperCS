import { useTranslation } from 'react-i18next';
import { StructureManager } from '@/features/manage-structure';
import { AdminLayout } from '@/widgets/admin-layout';

export function AdminStructurePage() {
    const { t } = useTranslation();
    return (
        <AdminLayout title={t('admin.structure.title')}>
            <StructureManager />
        </AdminLayout>
    );
}
