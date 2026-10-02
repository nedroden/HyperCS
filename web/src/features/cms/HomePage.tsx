import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { getHealth } from '../../api/client'

export function HomePage() {
  const { t } = useTranslation()
  const health = useQuery({ queryKey: ['health'], queryFn: getHealth, retry: false })
  const status = health.isPending ? 'loading' : health.isError ? 'down' : 'ok'

  return (
    <>
      <section className="hero">
        <h2>{t('home.title')}</h2>
        <p>{t('home.intro')}</p>
        <a className="btn" href="#">
          {t('home.learnMore')}
        </a>
      </section>
      <p className="api-status">{t(`api.${status}`)}</p>
    </>
  )
}
