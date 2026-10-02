type PaginationProps = { current: number; total: number };

/** Static pagination control; wire up navigation once list endpoints exist. */
export function Pagination({ current, total }: PaginationProps) {
    const pages = Array.from({ length: total }, (_, i) => i + 1);
    return (
        <nav className="pagination" aria-label="Pagination">
            {current > 1 ? (
                <a href="#">&laquo;</a>
            ) : (
                <span className="disabled">&laquo;</span>
            )}
            {pages.map((p) =>
                p === current ? (
                    <span key={p} className="current" aria-current="page">
                        {p}
                    </span>
                ) : (
                    <a key={p} href="#">
                        {p}
                    </a>
                ),
            )}
            {current < total ? (
                <a href="#">&raquo;</a>
            ) : (
                <span className="disabled">&raquo;</span>
            )}
        </nav>
    );
}
