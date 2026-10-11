import { useEffect, useId, useRef, useState, type KeyboardEvent } from 'react';
import { useTranslation } from 'react-i18next';
import { useLanguage } from '@/entities/session';
import { LANGUAGES } from '@/shared/i18n';
import { Flag } from '@/shared/ui';

export function LanguageSwitcher() {
    const { t } = useTranslation();
    const [open, setOpen] = useState(false);
    const root = useRef<HTMLDivElement>(null);
    const toggle = useRef<HTMLButtonElement>(null);
    const items = useRef<(HTMLButtonElement | null)[]>([]);
    const menuId = useId();
    const { language: current, setLanguage: select } = useLanguage();

    // Move focus into the menu (onto the current language) when it opens.
    useEffect(() => {
        if (!open) return;
        const index = LANGUAGES.findIndex((l) => l.code === current.code);
        items.current[Math.max(index, 0)]?.focus();
    }, [open, current.code]);

    useEffect(() => {
        if (!open) return;
        const onPointerDown = (e: MouseEvent) => {
            if (root.current && !root.current.contains(e.target as Node)) {
                setOpen(false);
            }
        };
        document.addEventListener('mousedown', onPointerDown);
        return () => document.removeEventListener('mousedown', onPointerDown);
    }, [open]);

    const closeAndFocusToggle = () => {
        setOpen(false);
        toggle.current?.focus();
    };

    const focusItem = (index: number) => {
        const count = LANGUAGES.length;
        items.current[(index + count) % count]?.focus();
    };

    const onToggleKeyDown = (e: KeyboardEvent) => {
        if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
            e.preventDefault();
            setOpen(true);
        }
    };

    const onMenuKeyDown = (e: KeyboardEvent) => {
        const index = items.current.findIndex(
            (el) => el === document.activeElement,
        );
        switch (e.key) {
            case 'Escape':
                e.preventDefault();
                e.stopPropagation();
                closeAndFocusToggle();
                break;
            case 'ArrowDown':
                e.preventDefault();
                focusItem(index + 1);
                break;
            case 'ArrowUp':
                e.preventDefault();
                focusItem(index - 1);
                break;
            case 'Home':
                e.preventDefault();
                focusItem(0);
                break;
            case 'End':
                e.preventDefault();
                focusItem(LANGUAGES.length - 1);
                break;
            case 'Tab':
                setOpen(false);
                break;
        }
    };

    return (
        <div className="lang" ref={root}>
            <button
                type="button"
                ref={toggle}
                className="lang-toggle"
                aria-haspopup="menu"
                aria-expanded={open}
                aria-controls={open ? menuId : undefined}
                aria-label={`${t('footer.language')}: ${current.label}`}
                onClick={() => setOpen((o) => !o)}
                onKeyDown={onToggleKeyDown}
            >
                <Flag code={current.flag} />
                {current.label}
            </button>
            {open && (
                <ul
                    className="lang-menu"
                    id={menuId}
                    role="menu"
                    aria-label={t('footer.language')}
                    onKeyDown={onMenuKeyDown}
                >
                    {LANGUAGES.map((l, i) => (
                        <li key={l.code} role="none">
                            <button
                                type="button"
                                role="menuitemradio"
                                aria-checked={l.code === current.code}
                                lang={l.code}
                                ref={(el) => {
                                    items.current[i] = el;
                                }}
                                onClick={() => {
                                    select(l.code);
                                    closeAndFocusToggle();
                                }}
                            >
                                <Flag code={l.flag} />
                                {l.label}
                            </button>
                        </li>
                    ))}
                </ul>
            )}
        </div>
    );
}
