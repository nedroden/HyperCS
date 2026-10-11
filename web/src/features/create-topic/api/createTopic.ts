import { api, unwrap, type Schema } from '@/shared/api';

/** `POST /boards/{id}/topics`: creates a topic (title + first post); resolves to the new topic's id. */
export async function createTopic(
    boardId: string,
    body: Schema<'CreateTopicRequest'>,
): Promise<string> {
    const created = await unwrap(
        api.POST('/api/v1/boards/{id}/topics', {
            params: { path: { id: boardId } },
            body,
        }),
    );
    return created.id;
}
