import type { i18n as I18n, TFunction } from 'i18next';

/**
 * Form error messages are i18n keys (`errors.<code>`), used both in zod schemas and for API field errors.
 * Unknown `errors.*` keys fall back to the generic message; anything else is shown as given.
 */
export function translateError(
    t: TFunction,
    i18n: I18n,
    message: string,
): string {
    if (i18n.exists(message)) return t(message);
    return message.startsWith('errors.') ? t('errors.unknown') : message;
}
