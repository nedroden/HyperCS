import { useTranslation } from 'react-i18next';
import { Link } from 'react-router-dom';
import { useRelativeTime } from '@/shared/lib';
import { Avatar } from '@/shared/ui';
import { categories, stats } from '../model/placeholder';

export function ForumIndexPage() {
    const { t } = useTranslation();
    const ago = useRelativeTime();
    return (
        <>
            <h2 className="page-title">{t('forum.boardIndex')}</h2>

            {categories.map((category) => (
                <section className="category" key={category.id}>
                    <div className="category-head">
                        <h3>{category.name}</h3>
                        <p>{category.description}</p>
                    </div>
                    {category.boards.map((board) => (
                        <div className="board" key={board.id}>
                            <div>
                                <Link
                                    className="name"
                                    to={`/forum/boards/${board.id}`}
                                >
                                    {board.name}
                                </Link>
                                <div className="desc">{board.description}</div>
                            </div>
                            <div className="stats">
                                {t('forum.topics', { count: board.topics })}
                                <small>
                                    {t('forum.posts', { count: board.posts })}
                                </small>
                            </div>
                            <div className="last">
                                {board.lastPost && (
                                    <>
                                        <Avatar
                                            name={board.lastPost.author}
                                            size="md"
                                            shape="round"
                                        />
                                        <div>
                                            <Link
                                                to={`/forum/topics/${board.lastPost.topicId}`}
                                            >
                                                {board.lastPost.title}
                                            </Link>
                                            <br />
                                            {t('forum.by')}{' '}
                                            <a href="#">
                                                {board.lastPost.author}
                                            </a>
                                            <br />
                                            {ago(board.lastPost.days)}
                                        </div>
                                    </>
                                )}
                            </div>
                        </div>
                    ))}
                </section>
            ))}

            <section className="category stats-section">
                <div className="category-head">
                    <h3>{t('forum.stats.title')}</h3>
                    <p>{t('forum.stats.intro')}</p>
                </div>
                <div className="stats-grid">
                    {(['topics', 'posts', 'members', 'boards'] as const).map(
                        (key) => (
                            <div className="stat" key={key}>
                                <div className="value">{stats[key]}</div>
                                <div className="label">
                                    {t(`forum.stats.${key}`)}
                                </div>
                            </div>
                        ),
                    )}
                </div>
                <div className="stats-extra">
                    {t('forum.stats.newestMember')}{' '}
                    <a href="#">{stats.newestMember}</a>
                    <br />
                    {t('forum.stats.latestPost')}{' '}
                    <Link to="/forum/topics/1">Lorem ipsum dolor sit amet</Link>{' '}
                    {t('forum.by')} <a href="#">{stats.newestMember}</a>,{' '}
                    {ago(7)}
                    <br />
                    {t('forum.stats.usersOnline', {
                        count: stats.usersOnline.length,
                    })}{' '}
                    {stats.usersOnline.map((u) => (
                        <a href="#" key={u}>
                            {u}
                        </a>
                    ))}
                </div>
            </section>
        </>
    );
}
