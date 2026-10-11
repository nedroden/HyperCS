import { zodResolver } from '@hookform/resolvers/zod';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { useTranslation } from 'react-i18next';
import type { HomepageContent } from '@/entities/cms-page';
import { problemOf } from '@/shared/api';
import { LIMITS } from '@/shared/config';
import { applyProblemToForm } from '@/shared/lib';
import { Field, FormErrorSummary } from '@/shared/ui';
import { heroSchema, type HeroValues } from '../model/schema';
import { useSaveHomepage } from '../model/useSave';
import { SavedNote } from './SavedNote';

const FIELDS = ['heroHeading', 'heroButtonLabel'] as const;

/** Hero section editor; saves the whole homepage document with the current feature blocks. */
export function HeroForm({ content }: { content: HomepageContent }) {
    const { t } = useTranslation();
    const save = useSaveHomepage();
    const [saved, setSaved] = useState(false);
    const {
        register,
        handleSubmit,
        setError,
        reset,
        formState: { errors, isSubmitting },
    } = useForm<HeroValues>({
        resolver: zodResolver(heroSchema),
        defaultValues: {
            heroHeading: content.heroHeading,
            heroButtonLabel: content.heroButtonLabel,
        },
    });
    const onSubmit = handleSubmit(async (values) => {
        setSaved(false);
        try {
            await save.mutateAsync({ ...content, ...values });
            reset(values);
            setSaved(true);
        } catch (error) {
            applyProblemToForm(problemOf(error), setError, FIELDS);
        }
    });
    return (
        <form onSubmit={onSubmit} noValidate>
            <section className="admin-panel" aria-labelledby="hero-heading">
                <h3 className="admin-panel-head" id="hero-heading">
                    {t('admin.homepage.hero')}
                </h3>
                <div className="admin-panel-body">
                    <FormErrorSummary errors={errors} />
                    <Field
                        id="heroHeading"
                        label={t('admin.homepage.heroHeading')}
                        error={errors.heroHeading}
                        required
                    >
                        {(control) => (
                            <input
                                type="text"
                                maxLength={LIMITS.heroHeading.max}
                                {...control}
                                {...register('heroHeading')}
                            />
                        )}
                    </Field>
                    <Field
                        id="heroButtonLabel"
                        label={t('admin.homepage.heroButton')}
                        error={errors.heroButtonLabel}
                        required
                    >
                        {(control) => (
                            <input
                                type="text"
                                maxLength={LIMITS.heroButton.max}
                                {...control}
                                {...register('heroButtonLabel')}
                            />
                        )}
                    </Field>
                    <button
                        className="btn"
                        type="submit"
                        disabled={isSubmitting}
                    >
                        {t('admin.homepage.saveHero')}
                    </button>
                    <SavedNote show={saved} />
                </div>
            </section>
        </form>
    );
}
