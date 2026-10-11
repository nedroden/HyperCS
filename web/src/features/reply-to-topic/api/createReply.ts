import { api, unwrap } from '@/shared/api';

/** `POST /topics/{id}/posts`: replies to a topic; resolves to the new post's id. */
export async function createReply(
    topicId: string,
    body: string,
): Promise<string> {
    const created = await unwrap(
        api.POST('/api/v1/topics/{id}/posts', {
            params: { path: { id: topicId } },
            body: { body },
        }),
    );
    return created.id;
}
