import { useTranslation } from 'react-i18next';
import { Link, NavLink } from 'react-router-dom';
import { useSession } from '@/entities/session';
import { useSiteSettings } from '@/entities/site-setting';
import { SignOutButton } from '@/features/sign-out';

export function Header() {
    const { t } = useTranslation();
    const { status, user, isAdmin } = useSession();
    const siteTitle = useSiteSettings().data?.siteTitle ?? t('app.name');
    // While `/me` loads (or failed) show neither the guest nor the member links, to avoid a flash.
    const known = status === 'authenticated' || status === 'guest';
    return (
        <>
            <header className="site-header">
                <h1>
                    <Link className="logo" to="/">
                        <img src="/logo.svg" alt={siteTitle} height={48} />
                    </Link>
                </h1>
                <div className="userbox">
                    {user ? (
                        <>
                            {t('userbox.hello')}{' '}
                            <Link to="/settings">
                                <strong>{user.displayName}</strong>
                            </Link>{' '}
                            |{' '}
                            {isAdmin && (
                                <>
                                    <Link to="/admin">
                                        {t('admin.nav.link')}
                                    </Link>{' '}
                                    |{' '}
                                </>
                            )}
                            <SignOutButton label={t('userbox.logout')} />
                        </>
                    ) : (
                        known && (
                            <>
                                <Link to="/login">{t('userbox.login')}</Link> |{' '}
                                <Link to="/register">
                                    {t('userbox.register')}
                                </Link>
                            </>
                        )
                    )}
                </div>
            </header>
            <nav className="site-nav" aria-label={t('a11y.mainNavigation')}>
                <NavLink to="/" end>
                    {t('nav.home')}
                </NavLink>
                <NavLink to="/forum">{t('nav.forum')}</NavLink>
                {user ? (
                    <SignOutButton label={t('nav.logout')} />
                ) : (
                    known && <NavLink to="/login">{t('nav.login')}</NavLink>
                )}
            </nav>
        </>
    );
}
