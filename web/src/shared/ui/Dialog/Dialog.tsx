import {
    useEffect,
    useId,
    useRef,
    type KeyboardEvent,
    type ReactNode,
} from 'react';
import { createPortal } from 'react-dom';

type DialogProps = {
    title: string;
    /** Called on Escape and on a click on the backdrop. */
    onClose: () => void;
    /** A wider dialog, e.g. for a Markdown editor. */
    wide?: boolean;
    children: ReactNode;
};

const FOCUSABLE =
    'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

/**
 * Modal dialog (`role="dialog"`, `aria-modal`): moves focus into the dialog, keeps Tab inside, closes on
 * Escape or a backdrop click and returns focus to the element that opened it.
 */
export function Dialog({ title, onClose, wide, children }: DialogProps) {
    const titleId = useId();
    const ref = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const previous = document.activeElement as HTMLElement | null;
        const root = ref.current;
        (
            root?.querySelector<HTMLElement>('[data-autofocus]') ??
            root?.querySelector<HTMLElement>(
                'input:not([disabled]), textarea:not([disabled]), select:not([disabled]), button:not([disabled])',
            ) ??
            root
        )?.focus();
        return () => previous?.focus?.();
    }, []);

    const onKeyDown = (e: KeyboardEvent) => {
        if (e.key === 'Escape') {
            e.stopPropagation();
            onClose();
            return;
        }
        if (e.key !== 'Tab' || !ref.current) return;
        const items = Array.from(
            ref.current.querySelectorAll<HTMLElement>(FOCUSABLE),
        );
        if (items.length === 0) return;
        const first = items[0];
        const last = items[items.length - 1];
        if (e.shiftKey && document.activeElement === first) {
            e.preventDefault();
            last.focus();
        } else if (!e.shiftKey && document.activeElement === last) {
            e.preventDefault();
            first.focus();
        }
    };

    return createPortal(
        <div
            className="dialog-backdrop"
            onMouseDown={(e) => {
                if (e.target === e.currentTarget) onClose();
            }}
        >
            <div
                className={wide ? 'dialog dialog-wide' : 'dialog'}
                role="dialog"
                aria-modal="true"
                aria-labelledby={titleId}
                tabIndex={-1}
                ref={ref}
                onKeyDown={onKeyDown}
            >
                <h3 className="dialog-title" id={titleId}>
                    {title}
                </h3>
                {children}
            </div>
        </div>,
        document.body,
    );
}
