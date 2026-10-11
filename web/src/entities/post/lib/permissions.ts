/** The part of the session the permission checks need (`useSession().user`). */
type Actor =
    { id: string; role: 'admin' | 'moderator' | 'member' } | null | undefined;

const isStaff = (user: Actor) =>
    user?.role === 'admin' || user?.role === 'moderator';

/** Authors, moderators and admins may edit a post. The API is the real gate; this only hides the controls. */
export function canEditPost(user: Actor, post: { author: { id: string } }) {
    return !!user && (user.id === post.author.id || isStaff(user));
}

export const canDeletePost = canEditPost;

/** Same roles as editing: they may open the history and restore a version. */
export const canViewPostHistory = canEditPost;
export const canRestoreRevision = canEditPost;
