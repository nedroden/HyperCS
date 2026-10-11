import { useTranslation } from 'react-i18next';
import { queryKeys, useApiMutation } from '@/shared/api';
import { LANGUAGES, type LanguageCode } from '@/shared/i18n';
import { updatePreferences } from '../api/session';
import { useSession } from './useSession';

/**
 * The single source of truth for the interface language: the i18n language (persisted to localStorage for
 * guests by `shared/i18n`); for a logged-in user the choice is also written to their profile. Used by the
 * footer switcher and the Preferences dropdown, so both always agree.
 */
export function useLanguage() {
    const { i18n } = useTranslation();
    const { user } = useSession();
    const save = useApiMutation({
        mutationFn: (language: LanguageCode) => updatePreferences({ language }),
        invalidates: [queryKeys.me.all],
    });
    const current =
        LANGUAGES.find((l) => l.code === i18n.language) ?? LANGUAGES[0];
    const setLanguage = (code: LanguageCode) => {
        void i18n.changeLanguage(code);
        if (user && user.language !== code) save.mutate(code);
    };
    return { language: current, setLanguage, error: save.error };
}
