import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import { DEFAULT_LANGUAGE, LANGUAGES, type LanguageCode } from './languages';
import { resourceFor } from './locales';

const STORAGE_KEY = 'hypercs.language';

function initialLanguage(): LanguageCode {
    const stored = localStorage.getItem(STORAGE_KEY);
    return LANGUAGES.find((l) => l.code === stored)?.code ?? DEFAULT_LANGUAGE;
}

/** Only deliberate changes are remembered: the language picked at startup is not a choice. */
let remember = false;

void i18n
    .use(initReactI18next)
    .init({
        resources: Object.fromEntries(
            LANGUAGES.map((l) => [
                l.code,
                { translation: resourceFor(l.code) },
            ]),
        ),
        lng: initialLanguage(),
        fallbackLng: DEFAULT_LANGUAGE,
        interpolation: { escapeValue: false },
    })
    .then(() => {
        remember = true;
    });

i18n.on('languageChanged', (lng) => {
    if (remember) localStorage.setItem(STORAGE_KEY, lng);
    document.documentElement.lang = lng;
});
document.documentElement.lang = i18n.language;

/**
 * Switches to the site's default language for a visitor who has never picked one; the switch itself is not
 * remembered, so a later change of the default still reaches them.
 */
export function applyDefaultLanguage(code: LanguageCode) {
    if (localStorage.getItem(STORAGE_KEY) !== null) return;
    if (i18n.language === code) return;
    remember = false;
    void i18n.changeLanguage(code).finally(() => {
        remember = true;
    });
}

export { i18n };
