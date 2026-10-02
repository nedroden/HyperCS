import { Fragment } from 'react';
import { Link } from 'react-router-dom';

export type Crumb = { label: string; to?: string };

export function Breadcrumbs({ items }: { items: Crumb[] }) {
    return (
        <nav className="breadcrumbs" aria-label="Breadcrumb">
            {items.map((item, i) => (
                <Fragment key={item.label}>
                    {i > 0 && <span aria-hidden="true">&rsaquo;</span>}
                    {item.to ? (
                        <Link to={item.to}>{item.label}</Link>
                    ) : (
                        item.label
                    )}
                </Fragment>
            ))}
        </nav>
    );
}
