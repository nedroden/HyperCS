import { zodResolver } from '@hookform/resolvers/zod';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { useTranslation } from 'react-i18next';
import { problemOf, queryKeys, useApiMutation } from '@/shared/api';
import { LIMITS } from '@/shared/config';
import { applyProblemToForm } from '@/shared/lib';
import { Field, FormErrorSummary, Panel } from '@/shared/ui';
import { changePassword } from '../api/changePassword';
import {
    changePasswordSchema,
    type ChangePasswordValues,
} from '../model/schema';

const FIELDS = ['currentPassword', 'newPassword', 'confirmPassword'] as const;
const EMPTY: ChangePasswordValues = {
    currentPassword: '',
    newPassword: '',
    confirmPassword: '',
};

/** The "Password" settings section. Other sessions are revoked by the server, so the sessions list is refreshed. */
export function ChangePasswordForm() {
    const { t } = useTranslation();
    const [done, setDone] = useState(false);
    const {
        register,
        handleSubmit,
        setError,
        reset,
        formState: { errors, isSubmitting },
    } = useForm<ChangePasswordValues>({
        resolver: zodResolver(changePasswordSchema),
        defaultValues: EMPTY,
    });
    const save = useApiMutation({
        mutationFn: (values: ChangePasswordValues) =>
            changePassword({
                currentPassword: values.currentPassword,
                newPassword: values.newPassword,
            }),
        invalidates: [queryKeys.me.sessions()],
    });

    const onSubmit = handleSubmit(async (values) => {
        setDone(false);
        try {
            await save.mutateAsync(values);
            reset(EMPTY);
            setDone(true);
        } catch (error) {
            applyProblemToForm(problemOf(error), setError, FIELDS);
        }
    });

    return (
        <form
            onSubmit={onSubmit}
            onChange={() => setDone(false)}
            noValidate
            aria-label={t('settings.password.title')}
        >
            <Panel
                id="password"
                title={t('settings.password.title')}
                intro={t('settings.password.intro')}
            >
                <FormErrorSummary errors={errors} />
                <Field
                    id="currentPassword"
                    label={t('settings.password.current')}
                    error={errors.currentPassword}
                    required
                >
                    {(control) => (
                        <input
                            type="password"
                            autoComplete="current-password"
                            maxLength={LIMITS.password.max}
                            {...control}
                            {...register('currentPassword')}
                        />
                    )}
                </Field>
                <div className="field-row">
                    <Field
                        id="newPassword"
                        label={t('settings.password.new')}
                        error={errors.newPassword}
                        hint={t('settings.password.hint', LIMITS.password)}
                        required
                    >
                        {(control) => (
                            <input
                                type="password"
                                autoComplete="new-password"
                                maxLength={LIMITS.password.max}
                                {...control}
                                {...register('newPassword')}
                            />
                        )}
                    </Field>
                    <Field
                        id="confirmPassword"
                        label={t('settings.password.confirm')}
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
                </div>
                <p className="hint form-note">
                    {t('settings.password.revokeNote')}
                </p>
                <button className="btn" type="submit" disabled={isSubmitting}>
                    {t('settings.password.save')}
                </button>
                <p className="form-status" role="status">
                    {done && t('settings.password.success')}
                </p>
            </Panel>
        </form>
    );
}
