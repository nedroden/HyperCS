import { useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { LANGUAGES } from '@/shared/i18n';
import { Flag } from '@/shared/ui';

export function LanguageSwitcher() {
    const { t, i18n } = useTranslation();
    const ref = useRef<HTMLDetailsElement>(null);
    const current =
        LANGUAGES.find((l) => l.code === i18n.language) ?? LANGUAGES[0];

    useEffect(() => {
        const close = () => ref.current?.removeAttribute('open');
        const onClick = (e: MouseEvent) => {
            if (ref.current && !ref.current.contains(e.target as Node)) close();
        };
        const onKey = (e: KeyboardEvent) => e.key === 'Escape' && close();
        document.addEventListener('click', onClick);
        document.addEventListener('keydown', onKey);
        return () => {
            document.removeEventListener('click', onClick);
            document.removeEventListener('keydown', onKey);
        };
    }, []);

    return (
        <details className="lang" ref={ref}>
            <summary aria-label={t('footer.language')}>
                <Flag code={current.flag} />
                {current.label}
            </summary>
            <ul className="lang-menu">
                {LANGUAGES.map((l) => (
                    <li key={l.code}>
                        <button
                            type="button"
                            aria-current={l.code === current.code}
                            onClick={() => {
                                void i18n.changeLanguage(l.code);
                                ref.current?.removeAttribute('open');
                            }}
                        >
                            <Flag code={l.flag} />
                            {l.label}
                        </button>
                    </li>
                ))}
            </ul>
        </details>
    );
}
