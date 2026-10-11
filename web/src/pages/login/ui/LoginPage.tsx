import { useTranslation } from 'react-i18next';
import { Link, Navigate, useLocation } from 'react-router-dom';
import { useSession } from '@/entities/session';
import { SignInForm } from '@/features/sign-in';
import { hasLoginRedirect, redirectTargetFrom } from '@/shared/lib';
import { FormBox } from '@/shared/ui';

export function LoginPage() {
    const { t } = useTranslation();
    const { status } = useSession();
    const { state } = useLocation();

    // Logged in (also right after a successful sign-in): go back where the user came from.
    if (status === 'authenticated') {
        return <Navigate to={redirectTargetFrom(state)} replace />;
    }

    return (
        <FormBox
            heading={t('auth.login.heading')}
            title={t('auth.login.title')}
            intro={
                hasLoginRedirect(state)
                    ? t('auth.login.required')
                    : t('auth.login.intro')
            }
            footer={
                <>
                    {t('auth.login.noAccount')}{' '}
                    <Link to="/register">{t('auth.login.register')}</Link>
                </>
            }
        >
            <SignInForm />
        </FormBox>
    );
}
