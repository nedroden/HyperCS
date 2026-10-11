import { lazy } from 'react';
import { Route, Routes } from 'react-router-dom';
import { MainLayout } from './layouts/MainLayout';
import { ErrorBoundary } from './lib/ErrorBoundary';
import { Providers } from './lib/Providers';
import { RequireAuth, RequireRole } from './guards';

/*
 * Route table. EVERY route is registered here with a lazily loaded page, so feature work only fills in the
 * page slice (`pages/<name>/ui`) and never edits this file. A page slice must keep exporting the component
 * named below from its `index.ts`.
 */
const HomePage = lazy(() =>
    import('@/pages/home').then((m) => ({ default: m.HomePage })),
);
const LoginPage = lazy(() =>
    import('@/pages/login').then((m) => ({ default: m.LoginPage })),
);
const RegisterPage = lazy(() =>
    import('@/pages/register').then((m) => ({ default: m.RegisterPage })),
);
const TermsPage = lazy(() =>
    import('@/pages/terms').then((m) => ({ default: m.TermsPage })),
);
const SettingsPage = lazy(() =>
    import('@/pages/settings').then((m) => ({ default: m.SettingsPage })),
);
const ForumIndexPage = lazy(() =>
    import('@/pages/forum-index').then((m) => ({ default: m.ForumIndexPage })),
);
const BoardPage = lazy(() =>
    import('@/pages/board').then((m) => ({ default: m.BoardPage })),
);
const NewTopicPage = lazy(() =>
    import('@/pages/new-topic').then((m) => ({ default: m.NewTopicPage })),
);
const TopicPage = lazy(() =>
    import('@/pages/topic').then((m) => ({ default: m.TopicPage })),
);
const EditPostPage = lazy(() =>
    import('@/pages/edit-post').then((m) => ({ default: m.EditPostPage })),
);
const PostHistoryPage = lazy(() =>
    import('@/pages/post-history').then((m) => ({
        default: m.PostHistoryPage,
    })),
);
const ProfilePage = lazy(() =>
    import('@/pages/profile').then((m) => ({ default: m.ProfilePage })),
);
const AdminStructurePage = lazy(() =>
    import('@/pages/admin-structure').then((m) => ({
        default: m.AdminStructurePage,
    })),
);
const AdminDashboardPage = lazy(() =>
    import('@/pages/admin-dashboard').then((m) => ({
        default: m.AdminDashboardPage,
    })),
);
const AdminSettingsPage = lazy(() =>
    import('@/pages/admin-settings').then((m) => ({
        default: m.AdminSettingsPage,
    })),
);
const AdminUsersPage = lazy(() =>
    import('@/pages/admin-users').then((m) => ({
        default: m.AdminUsersPage,
    })),
);
const AdminHomepagePage = lazy(() =>
    import('@/pages/admin-homepage').then((m) => ({
        default: m.AdminHomepagePage,
    })),
);
const AdminNewsPage = lazy(() =>
    import('@/pages/admin-news').then((m) => ({ default: m.AdminNewsPage })),
);
const NotFoundPage = lazy(() =>
    import('@/pages/not-found').then((m) => ({ default: m.NotFoundPage })),
);

export function AppRoutes() {
    return (
        <Routes>
            <Route element={<MainLayout />}>
                <Route index element={<HomePage />} />
                <Route path="login" element={<LoginPage />} />
                <Route path="register" element={<RegisterPage />} />
                <Route path="terms" element={<TermsPage />} />

                <Route path="forum" element={<ForumIndexPage />} />
                <Route
                    path="forum/boards/:boardId/:slug?"
                    element={<BoardPage />}
                />
                <Route
                    path="forum/topics/:topicId/:slug?"
                    element={<TopicPage />}
                />
                <Route path="forum/users/:username" element={<ProfilePage />} />

                {/* Logged-in users only */}
                <Route element={<RequireAuth />}>
                    <Route path="settings" element={<SettingsPage />} />
                    <Route
                        path="forum/boards/:boardId/new-topic"
                        element={<NewTopicPage />}
                    />
                    <Route
                        path="forum/posts/:postId/edit"
                        element={<EditPostPage />}
                    />
                    <Route
                        path="forum/posts/:postId/history"
                        element={<PostHistoryPage />}
                    />
                </Route>

                {/* Admins only */}
                <Route element={<RequireRole role="admin" />}>
                    <Route path="admin" element={<AdminDashboardPage />} />
                    <Route
                        path="admin/settings"
                        element={<AdminSettingsPage />}
                    />
                    <Route
                        path="admin/structure"
                        element={<AdminStructurePage />}
                    />
                    <Route path="admin/users" element={<AdminUsersPage />} />
                    <Route
                        path="admin/homepage"
                        element={<AdminHomepagePage />}
                    />
                    <Route path="admin/news" element={<AdminNewsPage />} />
                </Route>

                <Route path="*" element={<NotFoundPage />} />
            </Route>
        </Routes>
    );
}

export function App() {
    return (
        <ErrorBoundary>
            <Providers>
                <AppRoutes />
            </Providers>
        </ErrorBoundary>
    );
}
