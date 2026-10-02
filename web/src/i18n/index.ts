import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import de from './locales/de.json'
import enUS from './locales/en-US.json'

export const LANGUAGES = [
  { code: 'en-US', label: 'English (US)', flag: 'us' },
  { code: 'de', label: 'Deutsch', flag: 'de' },
] as const

export type LanguageCode = (typeof LANGUAGES)[number]['code']

const STORAGE_KEY = 'hypercs.language'

function initialLanguage(): LanguageCode {
  const stored = localStorage.getItem(STORAGE_KEY)
  return LANGUAGES.some((l) => l.code === stored) ? (stored as LanguageCode) : 'en-US'
}

void i18n.use(initReactI18next).init({
  resources: { 'en-US': { translation: enUS }, de: { translation: de } },
  lng: initialLanguage(),
  fallbackLng: 'en-US',
  interpolation: { escapeValue: false },
})

i18n.on('languageChanged', (lng) => {
  localStorage.setItem(STORAGE_KEY, lng)
  document.documentElement.lang = lng
})
document.documentElement.lang = i18n.language

export default i18n
