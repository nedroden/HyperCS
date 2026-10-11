import { render, screen } from '@testing-library/react';
import { expect, test } from 'vitest';
import { Markdown } from './Markdown';

test('renders basic markdown and GFM', () => {
    render(
        <Markdown>
            {
                '# Title\n\n**bold** and ~~gone~~\n\n- a\n- b\n\n| x |\n|---|\n| 1 |'
            }
        </Markdown>,
    );
    expect(screen.getByRole('heading', { name: 'Title' })).toBeInTheDocument();
    expect(screen.getByText('bold').tagName).toBe('STRONG');
    expect(screen.getByText('gone').tagName).toBe('DEL');
    expect(screen.getAllByRole('listitem')).toHaveLength(2);
    expect(screen.getByRole('table')).toBeInTheDocument();
});

test('does not render raw HTML', () => {
    const { container } = render(
        <Markdown>
            {
                '<script>alert(1)</script><img src=x onerror=alert(1)><b onclick="x()">raw</b>'
            }
        </Markdown>,
    );
    expect(container.querySelector('script')).toBeNull();
    expect(container.querySelector('img')).toBeNull();
    expect(container.querySelector('b')).toBeNull();
    expect(container.querySelector('[onclick]')).toBeNull();
});

test('strips dangerous link protocols', () => {
    render(<Markdown>{'[click](javascript:alert(1))'}</Markdown>);
    const el = screen.getByText('click');
    expect(el.getAttribute('href') ?? '').not.toMatch(/javascript:/i);
});

test('links get rel noopener nofollow ugc', () => {
    render(<Markdown>{'[site](https://example.com)'}</Markdown>);
    const link = screen.getByRole('link', { name: 'site' });
    expect(link).toHaveAttribute('href', 'https://example.com');
    expect(link).toHaveAttribute('rel', 'noopener nofollow ugc');
    expect(link).toHaveAttribute('target', '_blank');
});

test('images are shown as links, not loaded', () => {
    const { container } = render(
        <Markdown>{'![cat](https://example.com/cat.png)'}</Markdown>,
    );
    expect(container.querySelector('img')).toBeNull();
    expect(screen.getByRole('link', { name: 'cat' })).toHaveAttribute(
        'rel',
        'noopener nofollow ugc',
    );
});
