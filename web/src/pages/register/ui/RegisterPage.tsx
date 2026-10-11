import { useTranslation } from 'react-i18next';
import { Link, Navigate, useNavigate } from 'react-router-dom';
import { useSession } from '@/entities/session';
import { SignUpForm } from '@/features/sign-up';
import { FormBox } from '@/shared/ui';

export function RegisterPage() {
    const { t } = useTranslation();
    const { status } = useSession();
    const navigate = useNavigate();

    if (status === 'authenticated') return <Navigate to="/" replace />;

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
            <SignUpForm
                onSuccess={({ signedIn }) => {
                    // Signed in: this page redirects home by itself. Otherwise ask the user to log in.
                    if (!signedIn) navigate('/login');
                }}
            />
        </FormBox>
    );
}
