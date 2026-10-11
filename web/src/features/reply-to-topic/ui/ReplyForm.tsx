import { zodResolver } from '@hookform/resolvers/zod';
import { useEffect } from 'react';
import { Controller, useForm } from 'react-hook-form';
import { useTranslation } from 'react-i18next';
import { problemOf, queryKeys, useApiMutation } from '@/shared/api';
import { applyProblemToForm } from '@/shared/lib';
import { Field, FormErrorSummary, MarkdownEditor } from '@/shared/ui';
import { createReply } from '../api/createReply';
import { replySchema, type ReplyValues } from '../model/schema';

/** Text to append to the draft; a new `key` appends again (used by the quote button). */
export type ReplyInsertion = { key: number; text: string };

type ReplyFormProps = {
    topicId: string;
    topicTitle: string;
    /** Markdown inserted into the draft, e.g. a quoted post. */
    insertion?: ReplyInsertion;
    /** Called with the new post's id once the reply was created and the topic data refreshed. */
    onPosted: (postId: string) => void;
};

/** Inline reply form at the bottom of a topic with a live Markdown preview. The "Re: title" is derived here, not stored. */
export function ReplyForm({
    topicId,
    topicTitle,
    insertion,
    onPosted,
}: ReplyFormProps) {
    const { t } = useTranslation();
    const {
        control,
        handleSubmit,
        setError,
        setValue,
        getValues,
        reset,
        formState: { errors, isSubmitting },
    } = useForm<ReplyValues>({
        resolver: zodResolver(replySchema),
        defaultValues: { body: '' },
    });
    const reply = useApiMutation({
        mutationFn: (values: ReplyValues) => createReply(topicId, values.body),
        invalidates: [
            queryKeys.topics.all,
            queryKeys.boards.all,
            queryKeys.categories.all,
            queryKeys.stats.all,
        ],
    });

    const insertionKey = insertion?.key;
    useEffect(() => {
        if (insertionKey === undefined || !insertion) return;
        const current = getValues('body');
        const separator = current && !current.endsWith('\n\n') ? '\n\n' : '';
        setValue('body', `${current}${separator}${insertion.text}`, {
            shouldDirty: true,
        });
        const textarea = document.getElementById('reply-body');
        textarea?.focus();
        // Only a new insertion key triggers this, not re-renders with the same insertion.
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [insertionKey]);

    const onSubmit = handleSubmit(async (values) => {
        try {
            const postId = await reply.mutateAsync(values);
            reset({ body: '' });
            onPosted(postId);
        } catch (error) {
            // The draft stays in the editor so nothing is lost on a server error.
            applyProblemToForm(problemOf(error), setError, ['body']);
        }
    });

    return (
        <form onSubmit={onSubmit} noValidate>
            <FormErrorSummary errors={errors} />
            <p className="reply-subject">
                {t('forum.replySubject', { title: topicTitle })}
            </p>
            <Field
                id="reply-body"
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
                                placeholder={t('forum.replyPlaceholder')}
                                invalid={!!field['aria-invalid']}
                                describedBy={field['aria-describedby']}
                            />
                        )}
                    />
                )}
            </Field>
            <button className="btn" type="submit" disabled={isSubmitting}>
                {t('forum.submit')}
            </button>
        </form>
    );
}
