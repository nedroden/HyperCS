import { Component, type ErrorInfo, type ReactNode } from 'react';
import { useTranslation } from 'react-i18next';
import { ErrorState } from '@/shared/ui';

function Fallback() {
    const { t } = useTranslation();
    return (
        <div className="wrapper">
            <main className="site-main">
                <ErrorState
                    title={t('errorBoundary.title')}
                    text={t('errorBoundary.text')}
                    action={
                        <button
                            type="button"
                            className="btn"
                            onClick={() => window.location.reload()}
                        >
                            {t('errorBoundary.reload')}
                        </button>
                    }
                />
            </main>
        </div>
    );
}

/** Catches render errors anywhere below it and shows a reload prompt instead of a blank page. */
export class ErrorBoundary extends Component<
    { children: ReactNode },
    { failed: boolean }
> {
    state = { failed: false };

    static getDerivedStateFromError() {
        return { failed: true };
    }

    componentDidCatch(error: Error, info: ErrorInfo) {
        console.error(error, info.componentStack);
    }

    render() {
        return this.state.failed ? <Fallback /> : this.props.children;
    }
}
