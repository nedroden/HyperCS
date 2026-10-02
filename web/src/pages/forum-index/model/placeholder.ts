export type Board = {
    id: string;
    name: string;
    description: string;
    topics: number;
    posts: number;
    lastPost?: { title: string; topicId: string; author: string; days: number };
};

export type Category = {
    id: string;
    name: string;
    description: string;
    boards: Board[];
};

export const categories: Category[] = [
    {
        id: 'admin',
        name: 'Administration and Moderation',
        description:
            'Lorem ipsum dolor sit amet, consectetur adipiscing elit sed do eiusmod.',
        boards: [
            {
                id: '0',
                name: 'Forum administration',
                description:
                    'Ut enim ad minim veniam, quis nostrud exercitation.',
                topics: 0,
                posts: 0,
            },
        ],
    },
    {
        id: 'first',
        name: 'My first category',
        description: 'Duis aute irure dolor in reprehenderit in voluptate.',
        boards: [
            {
                id: '1',
                name: 'General Discussion',
                description: 'Excepteur sint occaecat cupidatat non proident.',
                topics: 1,
                posts: 1,
                lastPost: {
                    title: 'Lorem ipsum dolor sit amet',
                    topicId: '1',
                    author: 'Administrator',
                    days: 7,
                },
            },
            {
                id: '2',
                name: 'Second board',
                description: 'Sunt in culpa qui officia deserunt mollit anim.',
                topics: 0,
                posts: 0,
            },
        ],
    },
];

export const stats = {
    topics: 1,
    posts: 1,
    members: 1,
    boards: 3,
    newestMember: 'Administrator',
    usersOnline: ['Administrator'],
};
