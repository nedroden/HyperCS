import { useTranslation } from 'react-i18next';

/** "Pinned" / "Locked" badges shown before a topic title. */
export function TopicBadges({
    pinned,
    locked,
}: {
    pinned: boolean;
    locked: boolean;
}) {
    const { t } = useTranslation();
    return (
        <>
            {pinned && <span className="tag">{t('forum.pinned')}</span>}
            {locked && <span className="tag locked">{t('forum.locked')}</span>}
        </>
    );
}
