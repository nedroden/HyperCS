export type Post = {
    id: number;
    author: string;
    admin?: boolean;
    postCount: number;
    joinedDays: number;
    title: string;
    days: number;
    paragraphs: string[];
    canModerate: boolean;
};

export const topic = {
    id: '1',
    title: 'Lorem ipsum dolor sit amet',
    starter: 'Administrator',
    startedDays: 7,
    boardId: '1',
    boardName: 'General Discussion',
    category: 'My first category',
};

export const posts: Post[] = [
    {
        id: 1,
        author: 'Administrator',
        admin: true,
        postCount: 12,
        joinedDays: 7,
        title: topic.title,
        days: 7,
        canModerate: true,
        paragraphs: [
            'Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.',
            'Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.',
        ],
    },
    {
        id: 2,
        author: 'Jane',
        postCount: 4,
        joinedDays: 5,
        title: `Re: ${topic.title}`,
        days: 5,
        canModerate: false,
        paragraphs: [
            'Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur?',
        ],
    },
    {
        id: 3,
        author: 'Administrator',
        admin: true,
        postCount: 12,
        joinedDays: 7,
        title: `Re: ${topic.title}`,
        days: 4,
        canModerate: true,
        paragraphs: [
            'Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.',
        ],
    },
];
