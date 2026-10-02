export type Topic = {
    id: string;
    title: string;
    starter: string;
    startedDays: number;
    replies: number;
    views: number;
    lastPoster: string;
    lastDays: number;
    pinned?: boolean;
    locked?: boolean;
};

export const board = {
    id: '1',
    name: 'General Discussion',
    description: 'Excepteur sint occaecat cupidatat non proident.',
    category: 'My first category',
};

export const topics: Topic[] = [
    {
        id: '1',
        title: 'Lorem ipsum dolor sit amet',
        starter: 'Administrator',
        startedDays: 7,
        replies: 0,
        views: 42,
        lastPoster: 'Administrator',
        lastDays: 7,
        pinned: true,
    },
    {
        id: '1',
        title: 'Consectetur adipiscing elit',
        starter: 'Administrator',
        startedDays: 6,
        replies: 0,
        views: 31,
        lastPoster: 'Administrator',
        lastDays: 6,
        locked: true,
    },
    {
        id: '1',
        title: 'Sed do eiusmod tempor',
        starter: 'Jane',
        startedDays: 5,
        replies: 8,
        views: 120,
        lastPoster: 'Tom',
        lastDays: 2,
    },
    {
        id: '1',
        title: 'Ut labore et dolore magna',
        starter: 'Tom',
        startedDays: 4,
        replies: 3,
        views: 57,
        lastPoster: 'Administrator',
        lastDays: 1,
    },
    {
        id: '1',
        title: 'Quis nostrud exercitation ullamco?',
        starter: 'Jane',
        startedDays: 3,
        replies: 1,
        views: 19,
        lastPoster: 'Tom',
        lastDays: 0,
    },
];
