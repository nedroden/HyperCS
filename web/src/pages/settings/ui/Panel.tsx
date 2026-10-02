import type { FormEvent, ReactNode } from 'react';

type PanelProps = {
    id: string;
    title: string;
    intro: string;
    saveLabel: string;
    children: ReactNode;
};

/** One settings section: a form with its own save button. */
export function Panel({ id, title, intro, saveLabel, children }: PanelProps) {
    return (
        <form onSubmit={(e: FormEvent) => e.preventDefault()}>
            <section className="panel" id={id}>
                <div className="panel-head">
                    <h3>{title}</h3>
                    <p>{intro}</p>
                </div>
                <div className="panel-body">
                    {children}
                    <button className="btn" type="submit">
                        {saveLabel}
                    </button>
                </div>
            </section>
        </form>
    );
}
