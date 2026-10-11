export const LANGUAGES = [
    { code: 'en-US', label: 'English (US)', flag: 'us' },
    { code: 'de', label: 'Deutsch', flag: 'de' },
] as const;

export type LanguageCode = (typeof LANGUAGES)[number]['code'];
export const DEFAULT_LANGUAGE: LanguageCode = 'en-US';

export function isLanguageCode(value: string): value is LanguageCode {
    return LANGUAGES.some((l) => l.code === value);
}
