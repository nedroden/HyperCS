/** The part of the session the permission checks need (`useSession().user`). */
type Actor = { role: 'admin' | 'moderator' | 'member' } | null | undefined;

const isStaff = (user: Actor) =>
    user?.role === 'admin' || user?.role === 'moderator';

/** Members may reply; locked topics only accept replies from moderators and admins. The API enforces it too. */
export function canReplyToTopic(user: Actor, topic: { locked: boolean }) {
    return !!user && (!topic.locked || isStaff(user));
}

/** Pin, lock, move and delete a topic: moderators and admins. */
export function canModerateTopic(user: Actor) {
    return isStaff(user);
}
