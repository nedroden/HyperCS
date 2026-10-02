import type { ReactNode } from 'react';

type FormBoxProps = {
    heading: string;
    title: string;
    intro: string;
    footer?: ReactNode;
    children: ReactNode;
};

/** Centered box used by the login and registration pages. */
export function FormBox({
    heading,
    title,
    intro,
    footer,
    children,
}: FormBoxProps) {
    return (
        <div className="login-box">
            <h2 className="page-title">{heading}</h2>
            <div className="login-head">
                <h3>{title}</h3>
                <p>{intro}</p>
            </div>
            {children}
            {footer && <div className="login-alt">{footer}</div>}
        </div>
    );
}
