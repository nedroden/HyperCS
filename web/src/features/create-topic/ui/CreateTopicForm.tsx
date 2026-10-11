import { zodResolver } from '@hookform/resolvers/zod';
import { Controller, useForm } from 'react-hook-form';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { problemOf, queryKeys, useApiMutation } from '@/shared/api';
import { LIMITS } from '@/shared/config';
import { applyProblemToForm, boardPath, topicPath } from '@/shared/lib';
import { Field, FormErrorSummary, MarkdownEditor } from '@/shared/ui';
import { createTopic } from '../api/createTopic';
import { createTopicSchema, type CreateTopicValues } from '../model/schema';

const FIELDS = ['title', 'body'] as const;

/** New topic form (mocks/new-topic.html): title + Markdown message with live preview. Goes to the topic when done. */
export function CreateTopicForm({ boardId }: { boardId: string }) {
    const { t } = useTranslation();
    const navigate = useNavigate();
    const {
        register,
        control,
        handleSubmit,
        setError,
        formState: { errors, isSubmitting },
    } = useForm<CreateTopicValues>({
        resolver: zodResolver(createTopicSchema),
        defaultValues: { title: '', body: '' },
    });
    const create = useApiMutation({
        mutationFn: (values: CreateTopicValues) =>
            createTopic(boardId, { title: values.title, body: values.body }),
        invalidates: [
            queryKeys.boards.all,
            queryKeys.categories.all,
            queryKeys.topics.all,
            queryKeys.stats.all,
        ],
    });

    const onSubmit = handleSubmit(async (values) => {
        try {
            const id = await create.mutateAsync(values);
            navigate(topicPath(id));
        } catch (error) {
            // The typed text stays in the form so nothing is lost on a server error.
            applyProblemToForm(problemOf(error), setError, FIELDS);
        }
    });

    return (
        <form className="post-form" onSubmit={onSubmit} noValidate>
            <FormErrorSummary errors={errors} />
            <Field
                id="title"
                label={t('forum.form.title')}
                hint={t('forum.form.titleHint', {
                    min: LIMITS.topicTitle.min,
                    max: LIMITS.topicTitle.max,
                })}
                error={errors.title}
                required
            >
                {(field) => (
                    <input
                        type="text"
                        placeholder={t('forum.form.titlePlaceholder')}
                        maxLength={LIMITS.topicTitle.max}
                        {...field}
                        {...register('title')}
                    />
                )}
            </Field>
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
                                placeholder={t('forum.form.messagePlaceholder')}
                                invalid={!!field['aria-invalid']}
                                describedBy={field['aria-describedby']}
                            />
                        )}
                    />
                )}
            </Field>
            <div className="form-actions">
                <button className="btn" type="submit" disabled={isSubmitting}>
                    {t('forum.form.postTopic')}
                </button>
                <button
                    className="btn secondary"
                    type="button"
                    onClick={() => navigate(boardPath(boardId))}
                >
                    {t('forum.form.cancel')}
                </button>
            </div>
        </form>
    );
}
