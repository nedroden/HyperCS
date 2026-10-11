import { zodResolver } from '@hookform/resolvers/zod';
import { useState } from 'react';
import { useForm, useWatch } from 'react-hook-form';
import { useTranslation } from 'react-i18next';
import { useSession, type Me } from '@/entities/session';
import { problemOf, queryKeys, useApiMutation } from '@/shared/api';
import { LIMITS } from '@/shared/config';
import { applyProblemToForm, changedFields } from '@/shared/lib';
import { Avatar, Field, FormErrorSummary, Panel } from '@/shared/ui';
import { updateProfile, type ProfileUpdate } from '../api/updateProfile';
import {
    nullIfEmpty,
    profileSchema,
    type ProfileValues,
} from '../model/schema';

const FIELDS = ['displayName', 'location', 'website', 'aboutMe'] as const;

function valuesOf(user: Me): ProfileValues {
    return {
        displayName: user.displayName,
        location: user.location ?? '',
        website: user.website ?? '',
        aboutMe: user.aboutMe ?? '',
    };
}

/** The "Profile" settings section: public profile data, saved with a partial PATCH (changed fields only). */
export function ProfileForm() {
    const { user } = useSession();
    return user ? <ProfileFormFor user={user} /> : null;
}

function ProfileFormFor({ user }: { user: Me }) {
    const { t } = useTranslation();
    const [baseline, setBaseline] = useState(() => valuesOf(user));
    const [message, setMessage] = useState<'saved' | 'unchanged' | null>(null);
    const {
        register,
        handleSubmit,
        setError,
        reset,
        control,
        formState: { errors, isSubmitting },
    } = useForm<ProfileValues>({
        resolver: zodResolver(profileSchema),
        defaultValues: baseline,
    });
    const save = useApiMutation({
        mutationFn: (body: ProfileUpdate) => updateProfile(body),
        invalidates: [queryKeys.me.all],
    });

    const onSubmit = handleSubmit(async (values) => {
        setMessage(null);
        const changed = changedFields(values, baseline);
        if (Object.keys(changed).length === 0) {
            setMessage('unchanged');
            return;
        }
        const body: ProfileUpdate = {
            ...changed,
            ...(changed.location !== undefined && {
                location: nullIfEmpty(changed.location),
            }),
            ...(changed.website !== undefined && {
                website: nullIfEmpty(changed.website),
            }),
            ...(changed.aboutMe !== undefined && {
                aboutMe: nullIfEmpty(changed.aboutMe),
            }),
        };
        try {
            await save.mutateAsync(body);
            setBaseline(values);
            reset(values);
            setMessage('saved');
        } catch (error) {
            applyProblemToForm(problemOf(error), setError, FIELDS);
        }
    });

    const name = useWatch({ control, name: 'displayName' });
    return (
        <form
            onSubmit={onSubmit}
            onChange={() => setMessage(null)}
            noValidate
            aria-label={t('settings.profile.title')}
        >
            <Panel
                id="profile"
                title={t('settings.profile.title')}
                intro={t('settings.profile.intro')}
            >
                <FormErrorSummary errors={errors} />
                <div className="field">
                    <div className="field-label">
                        {t('settings.profile.avatar')}
                    </div>
                    <div className="avatar-row">
                        <Avatar name={name || user.displayName} size="lg" />
                        <div className="hint">
                            {t('settings.profile.avatarHint')}
                        </div>
                    </div>
                </div>
                <div className="field-row">
                    <Field
                        id="displayName"
                        label={t('settings.profile.displayName')}
                        error={errors.displayName}
                        required
                    >
                        {(control) => (
                            <input
                                type="text"
                                maxLength={LIMITS.displayName.max}
                                {...control}
                                {...register('displayName')}
                            />
                        )}
                    </Field>
                    <Field
                        id="location"
                        label={t('settings.profile.location')}
                        error={errors.location}
                    >
                        {(control) => (
                            <input
                                type="text"
                                maxLength={LIMITS.location.max}
                                {...control}
                                {...register('location')}
                            />
                        )}
                    </Field>
                </div>
                <Field
                    id="website"
                    label={t('settings.profile.website')}
                    error={errors.website}
                >
                    {(control) => (
                        <input
                            type="url"
                            placeholder="https://"
                            maxLength={LIMITS.website.max}
                            {...control}
                            {...register('website')}
                        />
                    )}
                </Field>
                <Field
                    id="aboutMe"
                    label={t('settings.profile.about')}
                    error={errors.aboutMe}
                    hint={t('settings.profile.aboutHint', {
                        max: LIMITS.aboutMe.max,
                    })}
                >
                    {(control) => (
                        <textarea
                            maxLength={LIMITS.aboutMe.max}
                            {...control}
                            {...register('aboutMe')}
                        />
                    )}
                </Field>
                <button className="btn" type="submit" disabled={isSubmitting}>
                    {t('settings.profile.save')}
                </button>
                <p className="form-status" role="status">
                    {message && t(`settings.${message}`)}
                </p>
            </Panel>
        </form>
    );
}
