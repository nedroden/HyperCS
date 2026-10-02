import type { FormEvent } from 'react';
import { useTranslation } from 'react-i18next';
import { Link, useNavigate } from 'react-router-dom';
import { useSession } from '@/entities/session';
import { FormBox } from '@/shared/ui';

export function LoginPage() {
    const { t } = useTranslation();
    const { login } = useSession();
    const navigate = useNavigate();

    function onSubmit(e: FormEvent) {
        e.preventDefault();
        login(); // placeholder until the API has authentication
        navigate('/');
    }

    return (
        <FormBox
            heading={t('auth.login.heading')}
            title={t('auth.login.title')}
            intro={t('auth.login.intro')}
            footer={
                <>
                    {t('auth.login.noAccount')}{' '}
                    <Link to="/register">{t('auth.login.register')}</Link>
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

                <label htmlFor="password">{t('auth.password')}</label>
                <input
                    type="password"
                    id="password"
                    name="password"
                    autoComplete="current-password"
                />

                <div className="form-row">
                    <span>
                        <input type="checkbox" id="remember" name="remember" />{' '}
                        <label htmlFor="remember" className="inline">
                            {t('auth.remember')}
                        </label>
                    </span>
                </div>

                <button className="btn btn-block" type="submit">
                    {t('auth.login.submit')}
                </button>
            </form>
        </FormBox>
    );
}
