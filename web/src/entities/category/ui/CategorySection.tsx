import { useTranslation } from 'react-i18next';
import { Link } from 'react-router-dom';
import { boardPath, topicPath, userPath } from '@/shared/lib';
import { Avatar, RelativeTime } from '@/shared/ui';
import type { CategoryWithBoards } from '../api/getCategories';

/** A category header with its boards: counters and last-post summary per board (mocks/board-index.html). */
export function CategorySection({
    category,
}: {
    category: CategoryWithBoards;
}) {
    const { t } = useTranslation();
    return (
        <section className="category" aria-labelledby={`cat-${category.id}`}>
            <div className="category-head">
                <h3 id={`cat-${category.id}`}>{category.name}</h3>
                {category.description && <p>{category.description}</p>}
            </div>
            {category.boards.map((board) => (
                <div className="board" key={board.id}>
                    <div>
                        <Link className="name" to={boardPath(board.id)}>
                            {board.name}
                        </Link>
                        {board.description && (
                            <div className="desc">{board.description}</div>
                        )}
                    </div>
                    <div className="stats">
                        {t('forum.topics', { count: board.topicCount })}
                        <small>
                            {t('forum.posts', { count: board.postCount })}
                        </small>
                    </div>
                    <div className="last">
                        {board.lastPost && (
                            <>
                                <Avatar
                                    name={board.lastPost.author.displayName}
                                    size="md"
                                    shape="round"
                                />
                                <div>
                                    <Link
                                        to={topicPath(board.lastPost.topicId)}
                                    >
                                        {board.lastPost.topicTitle}
                                    </Link>
                                    <br />
                                    {t('forum.by')}{' '}
                                    <Link
                                        to={userPath(
                                            board.lastPost.author.username,
                                        )}
                                    >
                                        {board.lastPost.author.displayName}
                                    </Link>
                                    <br />
                                    <RelativeTime
                                        date={board.lastPost.createdAt}
                                    />
                                </div>
                            </>
                        )}
                    </div>
                </div>
            ))}
        </section>
    );
}
