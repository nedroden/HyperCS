import { useTranslation } from 'react-i18next';
import { Link, useParams } from 'react-router-dom';
import { safeWebUrl, useUserProfile } from '@/entities/user';
import { ApiError } from '@/shared/api';
import { Avatar, ErrorState, Loading, Panel, RelativeTime } from '@/shared/ui';

/** Public member profile (`/forum/users/:username`). */
export function ProfilePage() {
    const { t } = useTranslation();
    const { username = '' } = useParams();
    const profile = useUserProfile(username);

    if (profile.isPending) return <Loading />;
    if (profile.isError) {
        const notFound =
            profile.error instanceof ApiError && profile.error.status === 404;
        return notFound ? (
            <ErrorState
                title={t('profile.notFound.title')}
                text={t('profile.notFound.text')}
                action={
                    <Link className="btn" to="/forum">
                        {t('profile.notFound.back')}
                    </Link>
                }
            />
        ) : (
            <ErrorState onRetry={() => void profile.refetch()} />
        );
    }

    const user = profile.data;
    const website = safeWebUrl(user.website);
    return (
        <>
            <div className="profile-head">
                <Avatar name={user.displayName} size="lg" />
                <div>
                    <h2>{user.displayName}</h2>
                    <div className="meta">
                        <span>{t(`profile.role.${user.role}`)}</span>
                        <span aria-hidden="true"> &middot; </span>
                        <span>
                            {t('profile.joined')}{' '}
                            <RelativeTime date={user.createdAt} />
                        </span>
                        <span aria-hidden="true"> &middot; </span>
                        <span>
                            {t('profile.posts', { count: user.postCount })}
                        </span>
                        {user.location && (
                            <>
                                <span aria-hidden="true"> &middot; </span>
                                <span>{user.location}</span>
                            </>
                        )}
                    </div>
                </div>
            </div>
            {website && (
                <p className="profile-website">
                    <a
                        href={website.href}
                        target="_blank"
                        rel="noopener noreferrer nofollow ugc"
                    >
                        {website.host}
                    </a>
                </p>
            )}
            {user.aboutMe && (
                <div className="profile-about">
                    <Panel id="about" title={t('profile.about')}>
                        <p className="profile-about-text">{user.aboutMe}</p>
                    </Panel>
                </div>
            )}
        </>
    );
}
