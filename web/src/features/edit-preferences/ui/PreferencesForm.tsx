import { zodResolver } from '@hookform/resolvers/zod';
import { useMemo, useState } from 'react';
import { useForm } from 'react-hook-form';
import { useTranslation } from 'react-i18next';
import {
    updatePreferences,
    useLanguage,
    useSession,
    type Me,
    type PreferencesUpdate,
} from '@/entities/session';
import { problemOf, queryKeys, useApiMutation } from '@/shared/api';
import { applyProblemToForm, changedFields } from '@/shared/lib';
import { LANGUAGES, type LanguageCode } from '@/shared/i18n';
import { CheckboxField, Field, FormErrorSummary, Panel } from '@/shared/ui';
import { preferencesSchema, type PreferencesValues } from '../model/schema';
import { timeZoneOptions } from '../model/timeZones';

const FIELDS = [
    'timezone',
    'notifyOnReply',
    'notifyOnQuote',
    'showOnline',
] as const;

function valuesOf(user: Me): PreferencesValues {
    return {
        timezone: user.timezone,
        notifyOnReply: user.notifyOnReply,
        notifyOnQuote: user.notifyOnQuote,
        showOnline: user.showOnline,
    };
}

/**
 * The "Preferences" settings section. The language dropdown is not part of the form: it is the same value
 * as the footer switcher (`useLanguage`) and applies immediately; the rest is saved with the button.
 */
export function PreferencesForm() {
    const { user } = useSession();
    return user ? <PreferencesFormFor user={user} /> : null;
}

function PreferencesFormFor({ user }: { user: Me }) {
    const { t } = useTranslation();
    const { language, setLanguage } = useLanguage();
    const [baseline, setBaseline] = useState(() => valuesOf(user));
    const [message, setMessage] = useState<'saved' | 'unchanged' | null>(null);
    const zones = useMemo(() => timeZoneOptions(user.timezone), [user]);
    const {
        register,
        handleSubmit,
        setError,
        reset,
        formState: { errors, isSubmitting },
    } = useForm<PreferencesValues>({
        resolver: zodResolver(preferencesSchema),
        defaultValues: baseline,
    });
    const save = useApiMutation({
        mutationFn: (body: PreferencesUpdate) => updatePreferences(body),
        invalidates: [queryKeys.me.all],
    });

    const onSubmit = handleSubmit(async (values) => {
        setMessage(null);
        const changed = changedFields(values, baseline);
        if (Object.keys(changed).length === 0) {
            setMessage('unchanged');
            return;
        }
        try {
            await save.mutateAsync(changed);
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
            aria-label={t('settings.preferences.title')}
        >
            <Panel
                id="preferences"
                title={t('settings.preferences.title')}
                intro={t('settings.preferences.intro')}
            >
                <FormErrorSummary errors={errors} />
                <div className="field-row">
                    <Field
                        id="language"
                        label={t('settings.preferences.language')}
                        hint={t('settings.preferences.languageHint')}
                    >
                        {(control) => (
                            <select
                                {...control}
                                value={language.code}
                                onChange={(e) =>
                                    setLanguage(e.target.value as LanguageCode)
                                }
                            >
                                {LANGUAGES.map((l) => (
                                    <option key={l.code} value={l.code}>
                                        {l.label}
                                    </option>
                                ))}
                            </select>
                        )}
                    </Field>
                    <Field
                        id="timezone"
                        label={t('settings.preferences.timezone')}
                        error={errors.timezone}
                    >
                        {(control) => (
                            <select {...control} {...register('timezone')}>
                                {zones.map((zone) => (
                                    <option key={zone} value={zone}>
                                        {zone}
                                    </option>
                                ))}
                            </select>
                        )}
                    </Field>
                </div>
                <fieldset className="field check-group">
                    <legend>{t('settings.preferences.notifications')}</legend>
                    <CheckboxField
                        id="notifyOnReply"
                        label={t('settings.preferences.notifyReply')}
                    >
                        {(control) => (
                            <input
                                type="checkbox"
                                {...control}
                                {...register('notifyOnReply')}
                            />
                        )}
                    </CheckboxField>
                    <CheckboxField
                        id="notifyOnQuote"
                        label={t('settings.preferences.notifyQuote')}
                    >
                        {(control) => (
                            <input
                                type="checkbox"
                                {...control}
                                {...register('notifyOnQuote')}
                            />
                        )}
                    </CheckboxField>
                    <div className="hint">
                        {t('settings.preferences.notifyHint')}
                    </div>
                </fieldset>
                <fieldset className="field check-group">
                    <legend>{t('settings.preferences.privacy')}</legend>
                    <CheckboxField
                        id="showOnline"
                        label={t('settings.preferences.showOnline')}
                    >
                        {(control) => (
                            <input
                                type="checkbox"
                                {...control}
                                {...register('showOnline')}
                            />
                        )}
                    </CheckboxField>
                </fieldset>
                <button className="btn" type="submit" disabled={isSubmitting}>
                    {t('settings.preferences.save')}
                </button>
                <p className="form-status" role="status">
                    {message && t(`settings.${message}`)}
                </p>
            </Panel>
        </form>
    );
}
