import { useTranslation } from 'react-i18next';
import { Link } from 'react-router-dom';
import { useSession } from '@/entities/session';
import {
    canDeletePost,
    canEditPost,
    canViewPostHistory,
    pageOfPost,
    PostCard,
} from '@/entities/post';
import { canReplyToTopic, type TopicDetail } from '@/entities/topic';
import { DeletePostButton } from '@/features/delete-post';
import { QuoteButton } from '@/features/quote-post';
import type { Schema } from '@/shared/api';
import { postEditPath, postHistoryPath } from '@/shared/lib';

type PostListProps = {
    topic: TopicDetail;
    posts: Schema<'Post'>[];
    /** Receives the Markdown blockquote of the quoted post. */
    onQuote: (markdown: string) => void;
};

/**
 * The posts of a topic page. Actions are shown only when the viewer may use them (author/moderator/admin
 * for edit, delete and history; anyone who may reply for quote); the API enforces the same rules.
 */
export function PostList({ topic, posts, onQuote }: PostListProps) {
    const { t } = useTranslation();
    const { user } = useSession();
    const mayReply = canReplyToTopic(user, topic);

    return (
        <div className="posts">
            {posts.map((post) => {
                const isFirst = post.id === topic.firstPostId;
                return (
                    <PostCard
                        key={post.id}
                        post={post}
                        title={
                            isFirst
                                ? topic.title
                                : t('forum.replySubject', {
                                      title: topic.title,
                                  })
                        }
                        permalink={{
                            topicId: topic.id,
                            slug: topic.slug,
                            page: pageOfPost(post.number),
                        }}
                        actions={
                            <>
                                {mayReply && (
                                    <QuoteButton
                                        post={post}
                                        onQuote={onQuote}
                                    />
                                )}
                                {canEditPost(user, post) && (
                                    <Link to={postEditPath(post.id)}>
                                        {t('forum.edit')}
                                    </Link>
                                )}
                                {post.editedAt &&
                                    canViewPostHistory(user, post) && (
                                        <Link to={postHistoryPath(post.id)}>
                                            {t('forum.history.link')}
                                        </Link>
                                    )}
                                {canDeletePost(user, post) && (
                                    <DeletePostButton
                                        postId={post.id}
                                        isFirstPost={isFirst}
                                        boardId={topic.boardId}
                                    />
                                )}
                            </>
                        }
                    />
                );
            })}
        </div>
    );
}
