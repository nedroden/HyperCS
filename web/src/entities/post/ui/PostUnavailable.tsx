import { useTranslation } from 'react-i18next';
import { Link } from 'react-router-dom';
import { ErrorState } from '@/shared/ui';

/** Shown when the viewer may not edit the post or see its history. */
export function PostForbidden() {
    const { t } = useTranslation();
    return (
        <ErrorState
            title={t('apiState.forbidden.title')}
            text={t('apiState.forbidden.text')}
            action={<Link to="/forum">{t('forum.boardIndex')}</Link>}
        />
    );
}
