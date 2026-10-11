import { LanguageSwitcher } from '@/features/switch-language';

/** The "Powered by" credit is a brand line: deliberately hard-coded in English, never translated. */
export function Footer() {
    return (
        <footer className="site-footer">
            <div>
                Powered by{' '}
                <a
                    href="https://robertmonden.com"
                    target="_blank"
                    rel="noopener noreferrer"
                >
                    HyperCS
                </a>{' '}
                | © 2026 Robert Monden
            </div>
            <LanguageSwitcher />
        </footer>
    );
}
