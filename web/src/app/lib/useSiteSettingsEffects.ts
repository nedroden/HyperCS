import { useEffect, useRef } from 'react';
import { useSession } from '@/entities/session';
import { useSiteSettings } from '@/entities/site-setting';
import { applyDefaultLanguage, isLanguageCode } from '@/shared/i18n';

/**
 * Applies the site settings to the page: the browser tab title, and the default language for visitors
 * who have not chosen one (logged-in users get their profile language instead).
 */
export function useSiteSettingsEffects() {
    const settings = useSiteSettings().data;
    const { status } = useSession();
    const shownTitle = useRef(document.title);

    useEffect(() => {
        if (!settings) return;
        // A page that set its own title (e.g. "Page not found") keeps it.
        if (document.title === shownTitle.current)
            document.title = settings.siteTitle;
        shownTitle.current = settings.siteTitle;
    }, [settings]);

    useEffect(() => {
        if (
            status === 'guest' &&
            settings &&
            isLanguageCode(settings.defaultLanguage)
        )
            applyDefaultLanguage(settings.defaultLanguage);
    }, [status, settings]);
}
