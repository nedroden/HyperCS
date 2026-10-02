import { Trans } from 'react-i18next';
import { LanguageSwitcher } from '@/features/switch-language';

export function Footer() {
    return (
        <footer className="site-footer">
            <div>
                <Trans
                    i18nKey="footer.poweredBy"
                    values={{ year: 2026 }}
                    components={{
                        link: (
                            <a
                                href="https://robertmonden.com"
                                target="_blank"
                                rel="noopener noreferrer"
                            />
                        ),
                    }}
                />
            </div>
            <LanguageSwitcher />
        </footer>
    );
}
