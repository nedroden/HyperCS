import { screen, within } from '@testing-library/react';
import { expect, test } from 'vitest';
import { makeMe, mockApi, problem, renderApp } from '../../../../test/utils';
import { HomePage } from './HomePage';

const HOMEPAGE = {
    heroHeading: 'Hello from the CMS',
    heroButtonLabel: 'Join us',
    features: [
        { title: 'Fast', text: 'Light stack.' },
        { title: 'Open', text: 'Open source.' },
        { title: 'Friendly', text: 'Welcoming.' },
    ],
};

function newsPage(items: { id: string; title: string; body: string }[]) {
    return {
        items: items.map((i) => ({
            ...i,
            slug: 'x',
            publishedAt: new Date(Date.now() - 12 * 86_400_000).toISOString(),
        })),
        page: 1,
        perPage: 5,
        total: items.length,
        totalPages: 1,
    };
}

test('shows the hero, three features and the latest news from the API', async () => {
    const mock = mockApi();
    mock.on('get', '/api/v1/cms/homepage', 200, HOMEPAGE);
    mock.on('get', '/api/v1/cms/site', 200, {
        siteTitle: 'My Club',
        slogan: 'An intro written by an admin.',
        defaultLanguage: 'en-US',
    });
    mock.on(
        'get',
        '/api/v1/news',
        200,
        newsPage([
            { id: '1', title: 'First post', body: 'Short news.' },
            { id: '2', title: 'Second post', body: 'x'.repeat(500) },
        ]),
    );
    renderApp(<HomePage />, { mock });

    expect(
        await screen.findByRole('heading', { name: 'Hello from the CMS' }),
    ).toBeInTheDocument();
    expect(screen.getByText('An intro written by an admin.')).toBeVisible();
    expect(screen.getByRole('link', { name: 'Join us' })).toHaveAttribute(
        'href',
        '/register',
    );
    expect(screen.getByRole('heading', { name: 'Fast' })).toBeInTheDocument();
    expect(
        screen.getByRole('heading', { name: 'Friendly' }),
    ).toBeInTheDocument();

    expect(
        await screen.findByRole('heading', { name: 'Latest news' }),
    ).toBeInTheDocument();
    const articles = await screen.findAllByRole('article');
    expect(articles).toHaveLength(2);
    expect(
        within(articles[0]).getByRole('heading', { name: 'First post' }),
    ).toBeInTheDocument();
    expect(within(articles[0]).getByText('12 days ago')).toBeInTheDocument();
    expect(within(articles[0]).getByText('Short news.')).toBeInTheDocument();
    expect(within(articles[0]).queryByRole('button')).toBeNull();
    // Long bodies are cut to an excerpt that can be expanded.
    await within(articles[1]).findByRole('button', { name: 'Read more' });
    expect(articles[1].textContent?.length).toBeLessThan(300);
    expect(mock.calls('get', '/api/v1/news')[0].search.get('perPage')).toBe(
        '5',
    );
});

test('never mentions the forum and has no leftover placeholder text', async () => {
    const mock = mockApi();
    mock.on('get', '/api/v1/cms/homepage', 200, HOMEPAGE);
    mock.on('get', '/api/v1/news', 200, newsPage([]));
    const { container } = renderApp(<HomePage />, { mock });
    await screen.findByText('There is no news yet.');
    expect(container.textContent).not.toMatch(/forum|lorem|api status/i);
});

test('hides the call-to-action button for logged-in members', async () => {
    const mock = mockApi();
    mock.on('get', '/api/v1/cms/homepage', 200, HOMEPAGE);
    mock.on('get', '/api/v1/news', 200, newsPage([]));
    renderApp(<HomePage />, { mock, me: makeMe() });
    await screen.findByRole('heading', { name: 'Hello from the CMS' });
    expect(screen.queryByRole('link', { name: 'Join us' })).toBeNull();
});

test('shows a loading state, then an error state with retry for each block', async () => {
    const mock = mockApi();
    mock.onUnlisted(
        'get',
        '/api/v1/cms/homepage',
        500,
        problem(500, 'internal_error'),
    );
    mock.onUnlisted('get', '/api/v1/news', 500, problem(500, 'internal_error'));
    const { user } = renderApp(<HomePage />, { mock });
    expect(screen.getAllByRole('status').length).toBeGreaterThan(0);
    expect(
        await screen.findByText('The homepage could not be loaded.'),
    ).toBeInTheDocument();
    expect(
        await screen.findByText('The news could not be loaded.'),
    ).toBeInTheDocument();

    mock.on('get', '/api/v1/cms/homepage', 200, HOMEPAGE);
    await user.click(screen.getAllByRole('button', { name: 'Try again' })[0]);
    expect(
        await screen.findByRole('heading', { name: 'Hello from the CMS' }),
    ).toBeInTheDocument();
});

test('expands a formatted news item to its full Markdown body', async () => {
    const mock = mockApi();
    mock.on('get', '/api/v1/cms/homepage', 200, HOMEPAGE);
    mock.on(
        'get',
        '/api/v1/news',
        200,
        newsPage([{ id: '1', title: 'Rich', body: 'Some **bold** text.' }]),
    );
    const { user } = renderApp(<HomePage />, { mock });
    expect(await screen.findByText('Some bold text.')).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Read more' }));
    expect(screen.getByText('bold').tagName).toBe('STRONG');
    await user.click(screen.getByRole('button', { name: 'Show less' }));
    expect(screen.getByText('Some bold text.')).toBeInTheDocument();
});
