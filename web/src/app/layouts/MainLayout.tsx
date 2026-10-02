import { Outlet } from 'react-router-dom';
import { Footer } from '@/widgets/footer';
import { Header } from '@/widgets/header';

export function MainLayout() {
    return (
        <div className="wrapper">
            <Header />
            <main className="site-main">
                <Outlet />
            </main>
            <Footer />
        </div>
    );
}
