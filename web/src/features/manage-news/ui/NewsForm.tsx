import { zodResolver } from '@hookform/resolvers/zod';
import { Controller, useForm } from 'react-hook-form';
import { useTranslation } from 'react-i18next';
import { problemOf } from '@/shared/api';
import { LIMITS } from '@/shared/config';
import { applyProblemToForm } from '@/shared/lib';
import { Dialog, Field, FormErrorSummary, MarkdownEditor } from '@/shared/ui';
import { newsSchema, type NewsValues } from '../model/schema';

type NewsFormProps = {
    title: string;
    initial?: NewsValues;
    /** Resolves when saved (the caller closes the dialog); throws an `ApiError` otherwise. */
    onSubmit: (values: NewsValues) => Promise<unknown>;
    onCancel: () => void;
};

/** Dialog with the news item form: title and a Markdown body with preview. */
export function NewsForm({
    title,
    initial,
    onSubmit,
    onCancel,
}: NewsFormProps) {
    const { t } = useTranslation();
    const {
        register,
        control,
        handleSubmit,
        setError,
        formState: { errors, isSubmitting },
    } = useForm<NewsValues>({
        resolver: zodResolver(newsSchema),
        defaultValues: initial ?? { title: '', body: '' },
    });
    const submit = handleSubmit(async (values) => {
        try {
            await onSubmit(values);
        } catch (error) {
            applyProblemToForm(problemOf(error), setError, ['title', 'body']);
        }
    });
    return (
        <Dialog title={title} onClose={onCancel} wide>
            <form onSubmit={submit} noValidate>
                <FormErrorSummary errors={errors} />
                <Field
                    id="title"
                    label={t('admin.news.titleLabel')}
                    error={errors.title}
                    required
                >
                    {(field) => (
                        <input
                            type="text"
                            maxLength={LIMITS.newsTitle.max}
                            {...field}
                            {...register('title')}
                        />
                    )}
                </Field>
                <Field
                    id="body"
                    label={t('admin.news.bodyLabel')}
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
                                    maxLength={LIMITS.newsBody.max}
                                    invalid={!!aria['aria-invalid']}
                                    describedBy={aria['aria-describedby']}
                                />
                            )}
                        />
                    )}
                </Field>
                <div className="dialog-actions">
                    <button
                        type="submit"
                        className="btn"
                        disabled={isSubmitting}
                    >
                        {isSubmitting
                            ? t('admin.actions.saving')
                            : t('admin.news.publish')}
                    </button>
                    <button
                        type="button"
                        className="btn secondary"
                        onClick={onCancel}
                        disabled={isSubmitting}
                    >
                        {t('admin.actions.cancel')}
                    </button>
                </div>
            </form>
        </Dialog>
    );
}
