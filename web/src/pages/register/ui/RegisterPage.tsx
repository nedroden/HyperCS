import type { FormEvent } from 'react';
import { Trans, useTranslation } from 'react-i18next';
import { Link, useNavigate } from 'react-router-dom';
import { FormBox } from '@/shared/ui';

export function RegisterPage() {
    const { t } = useTranslation();
    const navigate = useNavigate();

    function onSubmit(e: FormEvent) {
        e.preventDefault();
        navigate('/login'); // placeholder until the API has registration
    }

    return (
        <FormBox
            heading={t('auth.register.heading')}
            title={t('auth.register.title')}
            intro={t('auth.register.intro')}
            footer={
                <>
                    {t('auth.register.haveAccount')}{' '}
                    <Link to="/login">{t('auth.register.login')}</Link>
                </>
            }
        >
            <form onSubmit={onSubmit}>
                <label htmlFor="username">{t('auth.username')}</label>
                <input
                    type="text"
                    id="username"
                    name="username"
                    autoComplete="username"
                />

                <label htmlFor="email">{t('auth.email')}</label>
                <input
                    type="email"
                    id="email"
                    name="email"
                    autoComplete="email"
                />

                <label htmlFor="password">{t('auth.password')}</label>
                <input
                    type="password"
                    id="password"
                    name="password"
                    autoComplete="new-password"
                />

                <label htmlFor="password2">{t('auth.confirmPassword')}</label>
                <input
                    type="password"
                    id="password2"
                    name="password2"
                    autoComplete="new-password"
                />

                <div className="form-row">
                    <span>
                        <input type="checkbox" id="terms" name="terms" />{' '}
                        <label htmlFor="terms" className="inline">
                            <Trans
                                i18nKey="auth.register.terms"
                                components={{ link: <a href="#" /> }}
                            />
                        </label>
                    </span>
                </div>

                <button className="btn btn-block" type="submit">
                    {t('auth.register.submit')}
                </button>
            </form>
        </FormBox>
    );
}
