import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';
import { useTranslation } from 'react-i18next';
import { problemOf } from '@/shared/api';
import { LIMITS } from '@/shared/config';
import { applyProblemToForm } from '@/shared/lib';
import { CheckboxField, Dialog, Field, FormErrorSummary } from '@/shared/ui';
import { structureSchema, type StructureValues } from '../model/schema';

const FIELDS = ['name', 'description', 'adminOnly'] as const;

type StructureFormProps = {
    title: string;
    initial?: Partial<StructureValues>;
    /** Resolves when saved (the dialog is then closed by the caller); throws an `ApiError` otherwise. */
    onSubmit: (values: StructureValues) => Promise<unknown>;
    onCancel: () => void;
};

/** Dialog with the name / description / admin-only form shared by categories and boards. */
export function StructureForm({
    title,
    initial,
    onSubmit,
    onCancel,
}: StructureFormProps) {
    const { t } = useTranslation();
    const {
        register,
        handleSubmit,
        setError,
        formState: { errors, isSubmitting },
    } = useForm<StructureValues>({
        resolver: zodResolver(structureSchema),
        defaultValues: {
            name: initial?.name ?? '',
            description: initial?.description ?? '',
            adminOnly: initial?.adminOnly ?? false,
        },
    });
    const submit = handleSubmit(async (values) => {
        try {
            await onSubmit(values);
        } catch (error) {
            applyProblemToForm(problemOf(error), setError, FIELDS);
        }
    });
    return (
        <Dialog title={title} onClose={onCancel}>
            <form onSubmit={submit} noValidate>
                <FormErrorSummary errors={errors} />
                <Field
                    id="name"
                    label={t('admin.structure.name')}
                    error={errors.name}
                    required
                >
                    {(control) => (
                        <input
                            type="text"
                            maxLength={LIMITS.categoryBoardName.max}
                            {...control}
                            {...register('name')}
                        />
                    )}
                </Field>
                <Field
                    id="description"
                    label={t('admin.structure.description')}
                    error={errors.description}
                >
                    {(control) => (
                        <input
                            type="text"
                            maxLength={LIMITS.description.max}
                            {...control}
                            {...register('description')}
                        />
                    )}
                </Field>
                <CheckboxField
                    id="adminOnly"
                    label={t('admin.structure.adminOnlyLabel')}
                    error={errors.adminOnly}
                >
                    {(control) => (
                        <input
                            type="checkbox"
                            {...control}
                            {...register('adminOnly')}
                        />
                    )}
                </CheckboxField>
                <div className="dialog-actions">
                    <button
                        type="submit"
                        className="btn"
                        disabled={isSubmitting}
                    >
                        {isSubmitting
                            ? t('admin.actions.saving')
                            : t('admin.actions.save')}
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
