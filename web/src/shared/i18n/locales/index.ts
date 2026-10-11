import type { LanguageCode } from '../languages';

type Catalog = Record<string, unknown>;

const files = import.meta.glob<Catalog>('./*/*.json', {
    eager: true,
    import: 'default',
});

/**
 * Message catalogs, one JSON file per area under `locales/<language>/<namespace>.json`. Each file owns
 * its own top-level keys (`auth.json` -> `auth`, `forum.json` -> `forum`, ...), and the files of a language
 * are merged into one i18next `translation` resource, so keys stay `auth.login.heading` (no `ns:` prefix).
 * A test enforces that top-level keys are not defined in two files and that all languages match.
 */
export const catalogs: Record<string, Record<string, Catalog>> = {};

for (const [path, content] of Object.entries(files)) {
    const [, language, file] = /^\.\/([^/]+)\/([^/]+)\.json$/.exec(path) ?? [];
    if (!language) continue;
    (catalogs[language] ??= {})[file] = content;
}

export function resourceFor(language: LanguageCode): Catalog {
    return Object.assign({}, ...Object.values(catalogs[language] ?? {}));
}
