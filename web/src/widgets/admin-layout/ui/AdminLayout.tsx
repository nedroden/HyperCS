import type { ReactNode } from 'react';
import { useTranslation } from 'react-i18next';
import { NavLink } from 'react-router-dom';
import { Icon, type IconName } from '@/shared/ui';

/** The admin sections, in sidebar order. The dashboard tiles use the same list. */
export const ADMIN_SECTIONS: readonly {
    key: string;
    to: string;
    icon: IconName;
}[] = [
    { key: 'settings', to: '/admin/settings', icon: 'settings' },
    { key: 'structure', to: '/admin/structure', icon: 'structure' },
    { key: 'users', to: '/admin/users', icon: 'users' },
    { key: 'homepage', to: '/admin/homepage', icon: 'homepage' },
    { key: 'news', to: '/admin/news', icon: 'news' },
];

/** Shared frame of all admin screens: page title, a sidebar with the sections, and the screen itself. */
export function AdminLayout({
    title,
    children,
}: {
    title: string;
    children: ReactNode;
}) {
    const { t } = useTranslation();
    return (
        <>
            <h2 className="page-title">{title}</h2>
            <div className="admin-layout">
                <nav
                    className="admin-sidebar"
                    aria-label={t('admin.nav.label')}
                >
                    <NavLink to="/admin" end>
                        <Icon name="dashboard" />
                        {t('admin.nav.dashboard')}
                    </NavLink>
                    {ADMIN_SECTIONS.map((s) => (
                        <NavLink key={s.key} to={s.to}>
                            <Icon name={s.icon} />
                            {t(`admin.nav.${s.key}`)}
                        </NavLink>
                    ))}
                </nav>
                <div className="admin-content">{children}</div>
            </div>
        </>
    );
}
