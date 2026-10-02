import { Route, Routes } from 'react-router-dom';
import { BoardPage } from '@/pages/board';
import { ForumIndexPage } from '@/pages/forum-index';
import { HomePage } from '@/pages/home';
import { LoginPage } from '@/pages/login';
import { RegisterPage } from '@/pages/register';
import { SettingsPage } from '@/pages/settings';
import { TopicPage } from '@/pages/topic';
import { MainLayout } from './layouts/MainLayout';
import { Providers } from './lib/Providers';

export function App() {
    return (
        <Providers>
            <Routes>
                <Route element={<MainLayout />}>
                    <Route index element={<HomePage />} />
                    <Route path="login" element={<LoginPage />} />
                    <Route path="register" element={<RegisterPage />} />
                    <Route path="settings" element={<SettingsPage />} />
                    <Route path="forum" element={<ForumIndexPage />} />
                    <Route
                        path="forum/boards/:boardId"
                        element={<BoardPage />}
                    />
                    <Route
                        path="forum/topics/:topicId"
                        element={<TopicPage />}
                    />
                </Route>
            </Routes>
        </Providers>
    );
}
