import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import de from './locales/de.json';
import enUS from './locales/en-US.json';
import { DEFAULT_LANGUAGE, LANGUAGES, type LanguageCode } from './languages';

const STORAGE_KEY = 'hypercs.language';

function initialLanguage(): LanguageCode {
    const stored = localStorage.getItem(STORAGE_KEY);
    return LANGUAGES.find((l) => l.code === stored)?.code ?? DEFAULT_LANGUAGE;
}

void i18n.use(initReactI18next).init({
    resources: { 'en-US': { translation: enUS }, de: { translation: de } },
    lng: initialLanguage(),
    fallbackLng: DEFAULT_LANGUAGE,
    interpolation: { escapeValue: false },
});

i18n.on('languageChanged', (lng) => {
    localStorage.setItem(STORAGE_KEY, lng);
    document.documentElement.lang = lng;
});
document.documentElement.lang = i18n.language;

export { i18n };
