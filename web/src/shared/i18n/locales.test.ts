import { describe, expect, test } from 'vitest';
import { LANGUAGES } from './languages';
import { catalogs } from './locales';

type Catalog = Record<string, unknown>;

function keys(obj: Catalog, prefix = ''): string[] {
    return Object.entries(obj).flatMap(([k, v]) =>
        v && typeof v === 'object'
            ? keys(v as Catalog, `${prefix}${k}.`)
            : [`${prefix}${k}`],
    );
}

const languages = LANGUAGES.map((l) => l.code);
const namespaces = Object.keys(catalogs['en-US']).sort();

test('every language has the same namespace files', () => {
    for (const language of languages) {
        expect(Object.keys(catalogs[language]).sort()).toEqual(namespaces);
    }
});

describe.each(namespaces)('namespace %s', (ns) => {
    test('all languages have identical key sets', () => {
        const reference = keys(catalogs['en-US'][ns]).sort();
        for (const language of languages) {
            expect(keys(catalogs[language][ns]).sort()).toEqual(reference);
        }
    });

    test('plural keys come in _one/_other pairs', () => {
        for (const language of languages) {
            const all = keys(catalogs[language][ns]);
            for (const key of all.filter((k) => k.endsWith('_one'))) {
                expect(all).toContain(key.replace(/_one$/, '_other'));
            }
            for (const key of all.filter((k) => k.endsWith('_other'))) {
                expect(all).toContain(key.replace(/_other$/, '_one'));
            }
        }
    });
});

test('a top-level key is owned by exactly one namespace file', () => {
    for (const language of languages) {
        const owners = new Map<string, string>();
        for (const [ns, catalog] of Object.entries(catalogs[language])) {
            for (const key of Object.keys(catalog)) {
                expect(
                    owners.get(key),
                    `"${key}" is defined in ${owners.get(key)} and ${ns}`,
                ).toBeUndefined();
                owners.set(key, ns);
            }
        }
    }
});

test('error codes have a message in every language', () => {
    for (const language of languages) {
        const errors = (catalogs[language].errors as Catalog).errors as Record<
            string,
            unknown
        >;
        expect(Object.keys(errors).length).toBeGreaterThan(5);
        for (const value of Object.values(errors)) {
            expect(typeof value).toBe('string');
        }
    }
});
