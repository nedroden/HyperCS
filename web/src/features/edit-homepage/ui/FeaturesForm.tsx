import { zodResolver } from '@hookform/resolvers/zod';
import { useState } from 'react';
import { useForm, type Path } from 'react-hook-form';
import { useTranslation } from 'react-i18next';
import type { HomepageContent } from '@/entities/cms-page';
import { problemOf } from '@/shared/api';
import { LIMITS } from '@/shared/config';
import { applyProblemToForm } from '@/shared/lib';
import { Field, FormErrorSummary } from '@/shared/ui';
import {
    featuresSchema,
    formFieldName,
    type FeaturesValues,
} from '../model/schema';
import { useSaveHomepage } from '../model/useSave';
import { SavedNote } from './SavedNote';

/** Editor for the three feature blocks; saves the whole homepage document with the current hero. */
export function FeaturesForm({ content }: { content: HomepageContent }) {
    const { t } = useTranslation();
    const save = useSaveHomepage();
    const [saved, setSaved] = useState(false);
    const {
        register,
        handleSubmit,
        setError,
        reset,
        formState: { errors, isSubmitting },
    } = useForm<FeaturesValues>({
        resolver: zodResolver(featuresSchema),
        defaultValues: { features: content.features },
    });
    const onSubmit = handleSubmit(async (values) => {
        setSaved(false);
        try {
            await save.mutateAsync({ ...content, features: values.features });
            reset(values);
            setSaved(true);
        } catch (error) {
            const problem = problemOf(error);
            applyProblemToForm<FeaturesValues>(
                {
                    ...problem,
                    errors: problem.errors?.map((e) => ({
                        ...e,
                        field: formFieldName(e.field),
                    })),
                },
                setError,
            );
        }
    });
    const indexes = Array.from({ length: LIMITS.featureCount }, (_, i) => i);
    return (
        <form onSubmit={onSubmit} noValidate>
            <section className="admin-panel" aria-labelledby="features-heading">
                <h3 className="admin-panel-head" id="features-heading">
                    {t('admin.homepage.features')}
                </h3>
                <div className="admin-panel-body">
                    <FormErrorSummary errors={errors} />
                    {indexes.map((i) => (
                        <div className="feature-row" key={i}>
                            <Field
                                id={`features.${i}.title`}
                                label={t('admin.homepage.featureTitle', {
                                    n: i + 1,
                                })}
                                error={errors.features?.[i]?.title}
                                required
                            >
                                {(control) => (
                                    <input
                                        type="text"
                                        maxLength={LIMITS.featureTitle.max}
                                        {...control}
                                        {...register(
                                            `features.${i}.title` as Path<FeaturesValues>,
                                        )}
                                    />
                                )}
                            </Field>
                            <Field
                                id={`features.${i}.text`}
                                label={t('admin.homepage.featureText', {
                                    n: i + 1,
                                })}
                                error={errors.features?.[i]?.text}
                                required
                            >
                                {(control) => (
                                    <textarea
                                        className="admin-textarea"
                                        maxLength={LIMITS.featureText.max}
                                        {...control}
                                        {...register(
                                            `features.${i}.text` as Path<FeaturesValues>,
                                        )}
                                    />
                                )}
                            </Field>
                        </div>
                    ))}
                    <button
                        className="btn"
                        type="submit"
                        disabled={isSubmitting}
                    >
                        {t('admin.homepage.saveFeatures')}
                    </button>
                    <SavedNote show={saved} />
                </div>
            </section>
        </form>
    );
}
