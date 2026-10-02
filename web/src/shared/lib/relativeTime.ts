import { useTranslation } from 'react-i18next';

type Unit = 'hour' | 'day' | 'week' | 'month';

/** Returns a formatter for relative times in the current language, e.g. `ago(7, 'day')` -> "7 days ago". */
export function useRelativeTime() {
    const { i18n } = useTranslation();
    const rtf = new Intl.RelativeTimeFormat(i18n.language, {
        numeric: 'always',
    });
    return (amount: number, unit: Unit = 'day') => rtf.format(-amount, unit);
}
