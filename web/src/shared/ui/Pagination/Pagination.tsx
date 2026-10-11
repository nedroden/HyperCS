import { useTranslation } from 'react-i18next';
import { Link } from 'react-router-dom';

type PaginationProps = {
    /** Current page, 1-based. */
    current: number;
    /** Total number of pages. Renders nothing for fewer than two pages. */
    total: number;
    /** Link target for a page; defaults to `?page=N`. */
    getHref?: (page: number) => string;
};

type Item = number | 'gap-start' | 'gap-end';

const SIBLINGS = 2;

function items(current: number, total: number): Item[] {
    const pages = new Set<number>([1, total]);
    for (let p = current - SIBLINGS; p <= current + SIBLINGS; p++) {
        if (p >= 1 && p <= total) pages.add(p);
    }
    const sorted = [...pages].sort((a, b) => a - b);
    const result: Item[] = [];
    sorted.forEach((p, i) => {
        if (i > 0 && p - sorted[i - 1] > 1) {
            result.push(i === 1 ? 'gap-start' : 'gap-end');
        }
        result.push(p);
    });
    return result;
}

const defaultHref = (page: number) => `?page=${page}`;

/** Page-number navigation (« 1 2 3 »), built from real links so it is keyboard operable. */
export function Pagination({
    current,
    total,
    getHref = defaultHref,
}: PaginationProps) {
    const { t } = useTranslation();
    if (total < 2) return null;
    const page = Math.min(Math.max(current, 1), total);
    return (
        <nav className="pagination" aria-label={t('pagination.label')}>
            {page > 1 ? (
                <Link
                    to={getHref(page - 1)}
                    rel="prev"
                    aria-label={t('pagination.previous')}
                >
                    &laquo;
                </Link>
            ) : (
                <span className="disabled" aria-hidden="true">
                    &laquo;
                </span>
            )}
            {items(page, total).map((item) =>
                typeof item !== 'number' ? (
                    <span key={item} className="gap" aria-hidden="true">
                        &hellip;
                    </span>
                ) : item === page ? (
                    <span
                        key={item}
                        className="current"
                        aria-current="page"
                        aria-label={t('pagination.currentPage', { page: item })}
                    >
                        {item}
                    </span>
                ) : (
                    <Link
                        key={item}
                        to={getHref(item)}
                        aria-label={t('pagination.page', { page: item })}
                    >
                        {item}
                    </Link>
                ),
            )}
            {page < total ? (
                <Link
                    to={getHref(page + 1)}
                    rel="next"
                    aria-label={t('pagination.next')}
                >
                    &raquo;
                </Link>
            ) : (
                <span className="disabled" aria-hidden="true">
                    &raquo;
                </span>
            )}
        </nav>
    );
}
