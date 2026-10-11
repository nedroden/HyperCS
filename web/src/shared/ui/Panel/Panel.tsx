import type { ReactNode } from 'react';

type PanelProps = {
    /** Anchor id (settings sidebar links to it); also names the region. */
    id: string;
    title: string;
    intro?: string;
    children: ReactNode;
};

/** A titled section with a gray head and a bordered body (settings sections, profile "About me"). */
export function Panel({ id, title, intro, children }: PanelProps) {
    const headingId = `${id}-title`;
    return (
        <section className="panel" id={id} aria-labelledby={headingId}>
            <div className="panel-head">
                <h3 id={headingId}>{title}</h3>
                {intro && <p>{intro}</p>}
            </div>
            <div className="panel-body">{children}</div>
        </section>
    );
}
