import type { FieldErrors, FieldValues } from 'react-hook-form';
import { useTranslation } from 'react-i18next';
import { translateError } from './errorText';

type Item = { name: string; message: string };

function flatten(errors: unknown, prefix = ''): Item[] {
    if (!errors || typeof errors !== 'object') return [];
    const items: Item[] = [];
    for (const [key, value] of Object.entries(errors)) {
        if (key === 'ref' || key === 'types' || key === 'type') continue;
        const name = prefix ? `${prefix}.${key}` : key;
        if (value && typeof value === 'object') {
            const message = (value as { message?: unknown }).message;
            if (typeof message === 'string' && message) {
                items.push({ name, message });
            } else {
                items.push(...flatten(value, name));
            }
        }
    }
    return items;
}

/**
 * Error summary for a form (`role="alert"`, announced when it appears). Field errors link to their
 * control (by id = field name); `root.*` errors, e.g. from `applyProblemToForm`, are listed without a link.
 */
export function FormErrorSummary<T extends FieldValues>({
    errors,
}: {
    errors: FieldErrors<T>;
}) {
    const { t, i18n } = useTranslation();
    const items = flatten(errors);
    if (items.length === 0) return null;
    return (
        <div className="form-error-summary" role="alert">
            <p>{t('form.summary', { count: items.length })}</p>
            <ul>
                {items.map(({ name, message }) => (
                    <li key={name}>
                        {name.startsWith('root') ? (
                            translateError(t, i18n, message)
                        ) : (
                            <a
                                href={`#${name}`}
                                onClick={(e) => {
                                    e.preventDefault();
                                    document.getElementById(name)?.focus();
                                }}
                            >
                                {translateError(t, i18n, message)}
                            </a>
                        )}
                    </li>
                ))}
            </ul>
        </div>
    );
}
