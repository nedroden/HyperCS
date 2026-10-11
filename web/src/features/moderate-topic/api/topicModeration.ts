import { api, unwrap } from '@/shared/api';

/** `PATCH /topics/{id}` with `pinned` and/or `locked` (moderators and admins). */
export function setTopicFlags(
    topicId: string,
    flags: { pinned?: boolean; locked?: boolean },
): Promise<void> {
    return unwrap(
        api.PATCH('/api/v1/topics/{id}', {
            params: { path: { id: topicId } },
            body: flags,
        }),
    );
}

/** `DELETE /topics/{id}`: hard-deletes the topic with all its posts. */
export function deleteTopic(topicId: string): Promise<void> {
    return unwrap(
        api.DELETE('/api/v1/topics/{id}', {
            params: { path: { id: topicId } },
        }),
    );
}
