import { api, unwrap } from '@/shared/api';

/** `PATCH /topics/{id}` with `boardId`: moves the topic to another board (counters are fixed server-side). */
export function moveTopic(topicId: string, boardId: string): Promise<void> {
    return unwrap(
        api.PATCH('/api/v1/topics/{id}', {
            params: { path: { id: topicId } },
            body: { boardId },
        }),
    );
}
