import { useTranslation } from 'react-i18next';
import { Link, NavLink } from 'react-router-dom';
import { useSession } from '@/entities/session';

export function Header() {
    const { t } = useTranslation();
    const { user, logout } = useSession();
    return (
        <>
            <header className="site-header">
                <h1>
                    <Link className="logo" to="/">
                        <img src="/logo.svg" alt={t('app.name')} height={48} />
                    </Link>
                </h1>
                <div className="userbox">
                    {user ? (
                        <>
                            {t('userbox.hello')}{' '}
                            <Link to="/settings">
                                <strong>{user.name}</strong>
                            </Link>{' '}
                            |{' '}
                            <button
                                type="button"
                                className="link-button"
                                onClick={logout}
                            >
                                {t('userbox.logout')}
                            </button>
                        </>
                    ) : (
                        <>
                            <Link to="/login">{t('userbox.login')}</Link> |{' '}
                            <Link to="/register">{t('userbox.register')}</Link>
                        </>
                    )}
                </div>
            </header>
            <nav className="site-nav">
                <NavLink to="/" end>
                    {t('nav.home')}
                </NavLink>
                <NavLink to="/forum">{t('nav.forum')}</NavLink>
                {user ? (
                    <button
                        type="button"
                        className="link-button"
                        onClick={logout}
                    >
                        {t('nav.logout')}
                    </button>
                ) : (
                    <NavLink to="/login">{t('nav.login')}</NavLink>
                )}
            </nav>
        </>
    );
}
