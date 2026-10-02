import { Trans, useTranslation } from 'react-i18next'
import { Link, NavLink, Outlet } from 'react-router-dom'
import { LanguageSwitcher } from '../components/LanguageSwitcher'

export function Layout() {
  const { t } = useTranslation()
  return (
    <div className="wrapper">
      <header className="site-header">
        <h1>{t('app.name')}</h1>
        <div className="userbox">
          <Link to="/login">{t('userbox.login')}</Link> | <Link to="/register">{t('userbox.register')}</Link>
        </div>
      </header>
      <nav className="site-nav">
        <NavLink to="/">{t('nav.home')}</NavLink>
        <NavLink to="/forum">{t('nav.forum')}</NavLink>
        <NavLink to="/login">{t('nav.login')}</NavLink>
      </nav>
      <main className="site-main">
        <Outlet />
      </main>
      <footer className="site-footer">
        <div>
          <Trans i18nKey="footer.poweredBy" values={{ year: 2026 }} components={{ link: <a href="/" /> }} />
        </div>
        <LanguageSwitcher />
      </footer>
    </div>
  )
}
