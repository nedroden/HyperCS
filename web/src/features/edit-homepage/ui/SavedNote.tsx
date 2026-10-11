import { useTranslation } from 'react-i18next';

/** Live region confirming a save; rendered (empty) permanently so screen readers announce the change. */
export function SavedNote({ show }: { show: boolean }) {
    const { t } = useTranslation();
    return (
        <span className="admin-saved" role="status">
            {show ? t('admin.homepage.saved') : ''}
        </span>
    );
}
