import { zodResolver } from '@hookform/resolvers/zod';
import type { ReactNode } from 'react';
import { Controller, useForm } from 'react-hook-form';
import { useTranslation } from 'react-i18next';
import { problemOf, queryKeys, useApiMutation } from '@/shared/api';
import { applyProblemToForm } from '@/shared/lib';
import { Field, FormErrorSummary, MarkdownEditor } from '@/shared/ui';
import { updatePost } from '../api/updatePost';
import { editPostSchema, type EditPostValues } from '../model/schema';

type EditPostFormProps = {
    postId: string;
    initialBody: string;
    onSaved: () => void;
    onCancel: () => void;
    /** Extra content in the action row, e.g. the link to the edit history. */
    extra?: ReactNode;
};

/** Edit form of a post (mocks/edit-post.html) with Markdown preview. */
export function EditPostForm({
    postId,
    initialBody,
    onSaved,
    onCancel,
    extra,
}: EditPostFormProps) {
    const { t } = useTranslation();
    const {
        control,
        handleSubmit,
        setError,
        formState: { errors, isSubmitting },
    } = useForm<EditPostValues>({
        resolver: zodResolver(editPostSchema),
        defaultValues: { body: initialBody },
    });
    const save = useApiMutation({
        mutationFn: (values: EditPostValues) => updatePost(postId, values.body),
        invalidates: [queryKeys.topics.all, queryKeys.posts.all],
    });

    const onSubmit = handleSubmit(async (values) => {
        try {
            await save.mutateAsync(values);
            onSaved();
        } catch (error) {
            applyProblemToForm(problemOf(error), setError, ['body']);
        }
    });

    return (
        <form className="post-form" onSubmit={onSubmit} noValidate>
            <FormErrorSummary errors={errors} />
            <Field
                id="body"
                label={t('forum.form.message')}
                error={errors.body}
                required
            >
                {(field) => (
                    <Controller
                        control={control}
                        name="body"
                        render={({
                            field: { value, onChange, onBlur, name },
                        }) => (
                            <MarkdownEditor
                                id={field.id}
                                name={name}
                                value={value}
                                onChange={onChange}
                                onBlur={onBlur}
                                invalid={!!field['aria-invalid']}
                                describedBy={field['aria-describedby']}
                            />
                        )}
                    />
                )}
            </Field>
            <div className="form-actions">
                <button className="btn" type="submit" disabled={isSubmitting}>
                    {t('forum.form.saveChanges')}
                </button>
                <button
                    className="btn secondary"
                    type="button"
                    onClick={onCancel}
                >
                    {t('forum.form.cancel')}
                </button>
                {extra}
            </div>
        </form>
    );
}
