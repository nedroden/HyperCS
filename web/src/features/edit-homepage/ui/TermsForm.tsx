import { zodResolver } from '@hookform/resolvers/zod';
import { useState } from 'react';
import { Controller, useForm } from 'react-hook-form';
import { useTranslation } from 'react-i18next';
import type { TermsContent } from '@/entities/cms-page';
import { problemOf } from '@/shared/api';
import { LIMITS } from '@/shared/config';
import { applyProblemToForm } from '@/shared/lib';
import { Field, FormErrorSummary, MarkdownEditor } from '@/shared/ui';
import { termsSchema, type TermsValues } from '../model/schema';
import { useSaveTerms } from '../model/useSave';
import { SavedNote } from './SavedNote';

/** Markdown editor (with preview) for the terms of service page. */
export function TermsForm({ content }: { content: TermsContent }) {
    const { t } = useTranslation();
    const save = useSaveTerms();
    const [saved, setSaved] = useState(false);
    const {
        control,
        handleSubmit,
        setError,
        reset,
        formState: { errors, isSubmitting },
    } = useForm<TermsValues>({
        resolver: zodResolver(termsSchema),
        defaultValues: { body: content.body },
    });
    const onSubmit = handleSubmit(async (values) => {
        setSaved(false);
        try {
            await save.mutateAsync(values);
            reset(values);
            setSaved(true);
        } catch (error) {
            applyProblemToForm(problemOf(error), setError, ['body']);
        }
    });
    return (
        <form onSubmit={onSubmit} noValidate>
            <section className="admin-panel" aria-labelledby="terms-heading">
                <h3 className="admin-panel-head" id="terms-heading">
                    {t('admin.homepage.terms')}
                </h3>
                <div className="admin-panel-body">
                    <FormErrorSummary errors={errors} />
                    <Field
                        id="body"
                        label={t('admin.homepage.termsBody')}
                        error={errors.body}
                        required
                    >
                        {({ id, ...aria }) => (
                            <Controller
                                name="body"
                                control={control}
                                render={({ field }) => (
                                    <MarkdownEditor
                                        id={id}
                                        name={field.name}
                                        value={field.value}
                                        onChange={field.onChange}
                                        onBlur={field.onBlur}
                                        maxLength={LIMITS.termsBody.max}
                                        invalid={!!aria['aria-invalid']}
                                        describedBy={aria['aria-describedby']}
                                    />
                                )}
                            />
                        )}
                    </Field>
                    <button
                        className="btn"
                        type="submit"
                        disabled={isSubmitting}
                    >
                        {t('admin.homepage.saveTerms')}
                    </button>
                    <SavedNote show={saved} />
                </div>
            </section>
        </form>
    );
}
