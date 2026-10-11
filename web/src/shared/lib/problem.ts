import type { FieldValues, Path, UseFormSetError } from 'react-hook-form';

/** The parts of the API's `application/problem+json` body that form handling needs. */
export type FormProblem = {
    code?: string;
    errors?: { field: string; code: string; message?: string }[];
};

/** i18n key for an API error code: `errors.<code>`. */
export function errorKey(code: string): string {
    return `errors.${code}`;
}

/**
 * Maps `problem.errors[{field, code}]` onto react-hook-form fields. Each field error's `message` is the
 * i18n key `errors.<code>`; `Field` and `FormErrorSummary` translate it. Errors for fields the form does
 * not have, and the problem's own `code` when no field errors exist, are reported as `root.server`.
 * Returns the unmapped codes.
 */
export function applyProblemToForm<T extends FieldValues>(
    problem: FormProblem,
    setError: UseFormSetError<T>,
    knownFields?: readonly string[],
): string[] {
    const unmapped: string[] = [];
    for (const e of problem.errors ?? []) {
        if (!knownFields || knownFields.includes(e.field)) {
            setError(e.field as Path<T>, {
                type: 'server',
                message: errorKey(e.code),
            });
        } else {
            unmapped.push(e.code);
        }
    }
    if (!problem.errors?.length && problem.code) {
        unmapped.push(problem.code);
    }
    if (unmapped.length > 0) {
        setError('root.server', {
            type: 'server',
            message: errorKey(unmapped[0]),
        });
    }
    return unmapped;
}
