import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { ChangePasswordForm } from '@/features/change-password';
import { PreferencesForm } from '@/features/edit-preferences';
import { AccountForm, ProfileForm } from '@/features/edit-profile';
import { SessionsPanel } from '@/features/manage-sessions';

const SECTIONS = [
    'profile',
    'account',
    'password',
    'preferences',
    'sessions',
] as const;
type Section = (typeof SECTIONS)[number];

function sectionFromHash(hash: string): Section {
    const id = hash.replace('#', '');
    return SECTIONS.find((s) => s === id) ?? 'profile';
}

/** Account settings: one section per concern, each saved on its own. */
export function SettingsPage() {
    const { t } = useTranslation();
    const [active, setActive] = useState<Section>(() =>
        sectionFromHash(window.location.hash),
    );
    // Back/forward and links to `#section` change the hash without going through the tabs.
    useEffect(() => {
        const onHashChange = () =>
            setActive(sectionFromHash(window.location.hash));
        window.addEventListener('hashchange', onHashChange);
        return () => window.removeEventListener('hashchange', onHashChange);
    }, []);
    return (
        <>
            <h2 className="page-title">{t('settings.title')}</h2>
            <div className="settings-layout">
                <nav
                    className="settings-sidebar"
                    aria-label={t('settings.nav.label')}
                >
                    {SECTIONS.map((s) => (
                        <a
                            key={s}
                            className={s === active ? 'active' : undefined}
                            aria-current={s === active ? 'true' : undefined}
                            href={`#${s}`}
                            onClick={() => setActive(s)}
                        >
                            {t(`settings.nav.${s}`)}
                        </a>
                    ))}
                </nav>

                <div>
                    {active === 'profile' && <ProfileForm />}
                    {active === 'account' && <AccountForm />}
                    {active === 'password' && <ChangePasswordForm />}
                    {active === 'preferences' && <PreferencesForm />}
                    {active === 'sessions' && <SessionsPanel />}
                </div>
            </div>
        </>
    );
}
