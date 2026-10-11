import { zodResolver } from '@hookform/resolvers/zod';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { useTranslation } from 'react-i18next';
import {
    putSiteSettings,
    useSiteSettings,
    type SiteSettings,
} from '@/entities/site-setting';
import { problemOf, queryKeys, useApiMutation } from '@/shared/api';
import { LIMITS } from '@/shared/config';
import { LANGUAGES } from '@/shared/i18n';
import { applyProblemToForm } from '@/shared/lib';
import {
    ErrorState,
    Field,
    FormErrorSummary,
    Loading,
    Panel,
} from '@/shared/ui';
import { siteSettingsSchema, type SiteSettingsValues } from '../model/schema';

const FIELDS = ['siteTitle', 'slogan', 'defaultLanguage'] as const;

/** The "General settings" admin section: site title, slogan and default language. */
export function SiteSettingsForm() {
    const { t } = useTranslation();
    const settings = useSiteSettings();
    if (settings.isPending) return <Loading />;
    if (settings.isError)
        return (
            <ErrorState
                text={t('admin.settings.loadError')}
                onRetry={() => void settings.refetch()}
            />
        );
    return <SiteSettingsFormFor settings={settings.data} />;
}

function SiteSettingsFormFor({ settings }: { settings: SiteSettings }) {
    const { t } = useTranslation();
    const [saved, setSaved] = useState(false);
    const {
        register,
        handleSubmit,
        setError,
        reset,
        formState: { errors, isSubmitting },
    } = useForm<SiteSettingsValues>({
        resolver: zodResolver(siteSettingsSchema),
        defaultValues: settings,
    });
    const save = useApiMutation({
        mutationFn: (body: SiteSettings) => putSiteSettings(body),
        invalidates: [queryKeys.cms.site()],
    });

    const onSubmit = handleSubmit(async (values) => {
        setSaved(false);
        try {
            await save.mutateAsync(values);
            reset(values);
            setSaved(true);
        } catch (error) {
            applyProblemToForm(problemOf(error), setError, FIELDS);
        }
    });

    return (
        <form
            onSubmit={onSubmit}
            onChange={() => setSaved(false)}
            noValidate
            aria-label={t('admin.settings.title')}
        >
            <Panel
                id="general"
                title={t('admin.settings.title')}
                intro={t('admin.settings.intro')}
            >
                <FormErrorSummary errors={errors} />
                <Field
                    id="siteTitle"
                    label={t('admin.settings.siteTitle')}
                    hint={t('admin.settings.siteTitleHint')}
                    error={errors.siteTitle}
                    required
                >
                    {(control) => (
                        <input
                            type="text"
                            maxLength={LIMITS.siteTitle.max}
                            {...control}
                            {...register('siteTitle')}
                        />
                    )}
                </Field>
                <Field
                    id="slogan"
                    label={t('admin.settings.slogan')}
                    hint={t('admin.settings.sloganHint')}
                    error={errors.slogan}
                    required
                >
                    {(control) => (
                        <input
                            type="text"
                            maxLength={LIMITS.slogan.max}
                            {...control}
                            {...register('slogan')}
                        />
                    )}
                </Field>
                <Field
                    id="defaultLanguage"
                    label={t('admin.settings.defaultLanguage')}
                    hint={t('admin.settings.defaultLanguageHint')}
                    error={errors.defaultLanguage}
                >
                    {(control) => (
                        <select {...control} {...register('defaultLanguage')}>
                            {LANGUAGES.map((l) => (
                                <option key={l.code} value={l.code}>
                                    {l.label}
                                </option>
                            ))}
                        </select>
                    )}
                </Field>
                <button className="btn" type="submit" disabled={isSubmitting}>
                    {t('admin.settings.save')}
                </button>
                <p className="form-status" role="status">
                    {saved && t('settings.saved')}
                </p>
            </Panel>
        </form>
    );
}
