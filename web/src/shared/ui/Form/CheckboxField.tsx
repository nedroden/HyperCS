import type { ReactNode } from 'react';
import type { FieldError } from 'react-hook-form';
import { useTranslation } from 'react-i18next';
import { translateError } from './errorText';
import type { FieldControlProps } from './Field';

type CheckboxFieldProps = {
    /** Id of the checkbox; also the id the error summary links to. */
    id: string;
    label: ReactNode;
    error?: FieldError | string;
    /** Receives the props to spread onto the input: `<input type="checkbox" {...control} {...register('x')} />`. */
    children: (control: FieldControlProps) => ReactNode;
};

/** A checkbox with its inline label and an error message wired up via `aria-invalid` and `aria-describedby`. */
export function CheckboxField({
    id,
    label,
    error,
    children,
}: CheckboxFieldProps) {
    const { t, i18n } = useTranslation();
    const message = typeof error === 'string' ? error : error?.message;
    const errorId = `${id}-error`;
    return (
        <div className={message ? 'field field-invalid' : 'field'}>
            <span>
                {children({
                    id,
                    'aria-invalid': message ? true : undefined,
                    'aria-describedby': message ? errorId : undefined,
                })}{' '}
                <label htmlFor={id} className="inline">
                    {label}
                </label>
            </span>
            {message && (
                <p className="field-error" id={errorId}>
                    {translateError(t, i18n, message)}
                </p>
            )}
        </div>
    );
}
