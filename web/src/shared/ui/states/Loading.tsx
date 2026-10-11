import { useTranslation } from 'react-i18next';

/** Loading indicator announced politely to assistive technology. */
export function Loading({ label }: { label?: string }) {
    const { t } = useTranslation();
    return (
        <div className="state state-loading" role="status">
            <span className="spinner" aria-hidden="true" />
            <span>{label ?? t('common.loading')}</span>
        </div>
    );
}
