import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';
import { useTranslation } from 'react-i18next';
import { login, useSessionTransitions } from '@/entities/session';
import { problemOf, useApiMutation } from '@/shared/api';
import { applyProblemToForm } from '@/shared/lib';
import { CheckboxField, Field, FormErrorSummary } from '@/shared/ui';
import { signInSchema, type SignInValues } from '../model/schema';

const FIELDS = ['username', 'password', 'remember'] as const;

/** Username/password login form. A failed login shows the API's `invalid_credentials` as a form error. */
export function SignInForm({ onSuccess }: { onSuccess?: () => void }) {
    const { t } = useTranslation();
    const { afterLogin } = useSessionTransitions();
    const {
        register,
        handleSubmit,
        setError,
        formState: { errors, isSubmitting },
    } = useForm<SignInValues>({
        resolver: zodResolver(signInSchema),
        defaultValues: { username: '', password: '', remember: false },
    });
    const signIn = useApiMutation({
        mutationFn: async (values: SignInValues) => {
            await login(values);
            // The session was rotated: load the new account and CSRF token.
            await afterLogin();
        },
    });

    const onSubmit = handleSubmit(async (values) => {
        try {
            await signIn.mutateAsync(values);
            onSuccess?.();
        } catch (error) {
            applyProblemToForm(problemOf(error), setError, FIELDS);
        }
    });

    return (
        <form onSubmit={onSubmit} noValidate>
            <FormErrorSummary errors={errors} />
            <Field
                id="username"
                label={t('auth.username')}
                error={errors.username}
                required
            >
                {(control) => (
                    <input
                        type="text"
                        autoComplete="username"
                        {...control}
                        {...register('username')}
                    />
                )}
            </Field>
            <Field
                id="password"
                label={t('auth.password')}
                error={errors.password}
                required
            >
                {(control) => (
                    <input
                        type="password"
                        autoComplete="current-password"
                        {...control}
                        {...register('password')}
                    />
                )}
            </Field>
            <div className="form-row">
                <CheckboxField id="remember" label={t('auth.remember')}>
                    {(control) => (
                        <input
                            type="checkbox"
                            {...control}
                            {...register('remember')}
                        />
                    )}
                </CheckboxField>
            </div>
            <button
                className="btn btn-block"
                type="submit"
                disabled={isSubmitting}
            >
                {t('auth.login.submit')}
            </button>
        </form>
    );
}
