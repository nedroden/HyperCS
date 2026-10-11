import { zodResolver } from '@hookform/resolvers/zod';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { useTranslation } from 'react-i18next';
import { useSession, type Me } from '@/entities/session';
import { problemOf, queryKeys, useApiMutation } from '@/shared/api';
import { LIMITS } from '@/shared/config';
import { applyProblemToForm, changedFields } from '@/shared/lib';
import { Field, FormErrorSummary, Panel } from '@/shared/ui';
import { updateAccount, type AccountUpdate } from '../api/updateProfile';
import {
    accountSchema,
    nullIfEmpty,
    type AccountValues,
} from '../model/schema';

const FIELDS = ['firstName', 'lastName', 'email'] as const;

function valuesOf(user: Me): AccountValues {
    return {
        firstName: user.firstName ?? '',
        lastName: user.lastName ?? '',
        email: user.email,
    };
}

/** The "Account" settings section: private data (name, email). The username is shown read-only. */
export function AccountForm() {
    const { user } = useSession();
    return user ? <AccountFormFor user={user} /> : null;
}

function AccountFormFor({ user }: { user: Me }) {
    const { t } = useTranslation();
    const [baseline, setBaseline] = useState(() => valuesOf(user));
    const [message, setMessage] = useState<'saved' | 'unchanged' | null>(null);
    const {
        register,
        handleSubmit,
        setError,
        reset,
        formState: { errors, isSubmitting },
    } = useForm<AccountValues>({
        resolver: zodResolver(accountSchema),
        defaultValues: baseline,
    });
    const save = useApiMutation({
        mutationFn: (body: AccountUpdate) => updateAccount(body),
        invalidates: [queryKeys.me.all],
    });

    const onSubmit = handleSubmit(async (values) => {
        setMessage(null);
        const changed = changedFields(values, baseline);
        if (Object.keys(changed).length === 0) {
            setMessage('unchanged');
            return;
        }
        const body: AccountUpdate = {
            ...changed,
            ...(changed.firstName !== undefined && {
                firstName: nullIfEmpty(changed.firstName),
            }),
            ...(changed.lastName !== undefined && {
                lastName: nullIfEmpty(changed.lastName),
            }),
        };
        try {
            await save.mutateAsync(body);
            setBaseline(values);
            reset(values);
            setMessage('saved');
        } catch (error) {
            applyProblemToForm(problemOf(error), setError, FIELDS);
        }
    });

    return (
        <form
            onSubmit={onSubmit}
            onChange={() => setMessage(null)}
            noValidate
            aria-label={t('settings.account.title')}
        >
            <Panel
                id="account"
                title={t('settings.account.title')}
                intro={t('settings.account.intro')}
            >
                <FormErrorSummary errors={errors} />
                <div className="field-row">
                    <Field
                        id="firstName"
                        label={t('settings.account.firstName')}
                        error={errors.firstName}
                    >
                        {(control) => (
                            <input
                                type="text"
                                autoComplete="given-name"
                                maxLength={LIMITS.personName.max}
                                {...control}
                                {...register('firstName')}
                            />
                        )}
                    </Field>
                    <Field
                        id="lastName"
                        label={t('settings.account.lastName')}
                        error={errors.lastName}
                    >
                        {(control) => (
                            <input
                                type="text"
                                autoComplete="family-name"
                                maxLength={LIMITS.personName.max}
                                {...control}
                                {...register('lastName')}
                            />
                        )}
                    </Field>
                </div>
                <Field
                    id="username"
                    label={t('settings.account.username')}
                    hint={t('settings.account.usernameHint')}
                >
                    {(control) => (
                        <input
                            type="text"
                            value={user.username}
                            readOnly
                            disabled
                            {...control}
                        />
                    )}
                </Field>
                <Field
                    id="email"
                    label={t('settings.account.email')}
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
                <button className="btn" type="submit" disabled={isSubmitting}>
                    {t('settings.account.save')}
                </button>
                <p className="form-status" role="status">
                    {message && t(`settings.${message}`)}
                </p>
            </Panel>
        </form>
    );
}
