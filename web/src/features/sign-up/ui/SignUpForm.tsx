import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';
import { Trans, useTranslation } from 'react-i18next';
import { Link } from 'react-router-dom';
import {
    fetchMe,
    login,
    updatePreferences,
    useSessionTransitions,
} from '@/entities/session';
import { problemOf, useApiMutation } from '@/shared/api';
import { LIMITS } from '@/shared/config';
import { i18n, isLanguageCode } from '@/shared/i18n';
import { applyProblemToForm } from '@/shared/lib';
import { CheckboxField, Field, FormErrorSummary } from '@/shared/ui';
import { register as registerAccount } from '../api/register';
import { browserTimeZone } from '../model/browserTimeZone';
import { signUpSchema, type SignUpValues } from '../model/schema';

const FIELDS = ['username', 'email', 'password', 'acceptTerms'] as const;

type SignUpFormProps = {
    /** Called after the account was created; `signedIn` is false if the automatic login failed. */
    onSuccess?: (result: { signedIn: boolean }) => void;
};

/**
 * Registration form. On success the new member is logged in right away (the API does not do that) and
 * their language and the browser's time zone are stored as their preferences (best effort).
 */
export function SignUpForm({ onSuccess }: SignUpFormProps) {
    const { t } = useTranslation();
    const { afterLogin } = useSessionTransitions();
    const {
        register,
        handleSubmit,
        setError,
        formState: { errors, isSubmitting },
    } = useForm<SignUpValues>({
        resolver: zodResolver(signUpSchema),
        defaultValues: {
            username: '',
            email: '',
            password: '',
            confirmPassword: '',
            acceptTerms: false,
        },
    });
    const signUp = useApiMutation({
        mutationFn: async (values: SignUpValues) => {
            const language = i18n.language;
            await registerAccount({
                username: values.username,
                email: values.email,
                password: values.password,
                acceptTerms: values.acceptTerms,
            });
            try {
                await login({
                    username: values.username,
                    password: values.password,
                });
                // Prime the CSRF token, store preferences, and only then publish the user to the cache so the
                // language applied on login is the one the visitor already chose.
                await fetchMe();
                try {
                    await updatePreferences({
                        timezone: browserTimeZone(),
                        language: isLanguageCode(language)
                            ? language
                            : undefined,
                    });
                } catch {
                    // Preferences are a convenience; the account exists and the user can change them in settings.
                }
                await afterLogin();
                return { signedIn: true };
            } catch {
                return { signedIn: false };
            }
        },
    });

    const onSubmit = handleSubmit(async (values) => {
        try {
            // Await first: `onSuccess?.(await ...)` would skip the call entirely when no callback is given.
            const result = await signUp.mutateAsync(values);
            onSuccess?.(result);
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
                hint={t('auth.usernameHint', LIMITS.username)}
                required
            >
                {(control) => (
                    <input
                        type="text"
                        autoComplete="username"
                        maxLength={LIMITS.username.max}
                        {...control}
                        {...register('username')}
                    />
                )}
            </Field>
            <Field
                id="email"
                label={t('auth.email')}
                error={errors.email}
                required
            >
                {(control) => (
                    <input
                        type="email"
                        autoComplete="email"
                        maxLength={LIMITS.email.max}
                        {...control}
                        {...register('email')}
                    />
                )}
            </Field>
            <Field
                id="password"
                label={t('auth.password')}
                error={errors.password}
                hint={t('auth.passwordHint', LIMITS.password)}
                required
            >
                {(control) => (
                    <input
                        type="password"
                        autoComplete="new-password"
                        maxLength={LIMITS.password.max}
                        {...control}
                        {...register('password')}
                    />
                )}
            </Field>
            <Field
                id="confirmPassword"
                label={t('auth.confirmPassword')}
                error={errors.confirmPassword}
                required
            >
                {(control) => (
                    <input
                        type="password"
                        autoComplete="new-password"
                        maxLength={LIMITS.password.max}
                        {...control}
                        {...register('confirmPassword')}
                    />
                )}
            </Field>
            <div className="form-row">
                <CheckboxField
                    id="acceptTerms"
                    error={errors.acceptTerms}
                    label={
                        <Trans
                            i18nKey="auth.register.terms"
                            components={{
                                terms: <Link to="/terms" target="_blank" />,
                            }}
                        />
                    }
                >
                    {(control) => (
                        <input
                            type="checkbox"
                            {...control}
                            {...register('acceptTerms')}
                        />
                    )}
                </CheckboxField>
            </div>
            <button
                className="btn btn-block"
                type="submit"
                disabled={isSubmitting}
            >
                {t('auth.register.submit')}
            </button>
        </form>
    );
}
