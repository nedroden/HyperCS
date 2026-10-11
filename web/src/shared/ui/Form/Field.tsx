import type { ReactNode } from 'react';
import type { FieldError } from 'react-hook-form';
import { useTranslation } from 'react-i18next';
import { translateError } from './errorText';

export type FieldControlProps = {
    id: string;
    'aria-invalid': boolean | undefined;
    'aria-describedby': string | undefined;
    'aria-required'?: boolean;
};

type FieldProps = {
    /** Id of the control; also the id the error summary links to. */
    id: string;
    label: ReactNode;
    /** An react-hook-form `FieldError` or an i18n key such as `errors.required`. */
    error?: FieldError | string;
    hint?: ReactNode;
    required?: boolean;
    /** Receives the props to spread onto the control, e.g. `<input {...control} {...register('name')} />`. */
    children: (control: FieldControlProps) => ReactNode;
};

/** Labelled form field with hint and error text wired up via `aria-invalid` and `aria-describedby`. */
export function Field({
    id,
    label,
    error,
    hint,
    required,
    children,
}: FieldProps) {
    const { t, i18n } = useTranslation();
    const message = typeof error === 'string' ? error : error?.message;
    const hintId = `${id}-hint`;
    const errorId = `${id}-error`;
    const describedBy =
        [hint ? hintId : undefined, message ? errorId : undefined]
            .filter(Boolean)
            .join(' ') || undefined;
    return (
        <div className={message ? 'field field-invalid' : 'field'}>
            <label htmlFor={id}>{label}</label>
            {children({
                id,
                'aria-invalid': message ? true : undefined,
                'aria-describedby': describedBy,
                ...(required ? { 'aria-required': true } : {}),
            })}
            {hint && (
                <div className="hint" id={hintId}>
                    {hint}
                </div>
            )}
            {message && (
                <p className="field-error" id={errorId}>
                    {translateError(t, i18n, message)}
                </p>
            )}
        </div>
    );
}
