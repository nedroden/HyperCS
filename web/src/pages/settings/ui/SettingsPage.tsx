import { useTranslation } from 'react-i18next';
import { Avatar } from '@/shared/ui';
import { Panel } from './Panel';

const SECTIONS = ['profile', 'account', 'password', 'preferences'] as const;

export function SettingsPage() {
    const { t } = useTranslation();
    return (
        <>
            <h2 className="page-title">{t('settings.title')}</h2>
            <div className="settings-layout">
                <aside className="settings-sidebar">
                    {SECTIONS.map((s, i) => (
                        <a
                            key={s}
                            className={i === 0 ? 'active' : undefined}
                            href={`#${s}`}
                        >
                            {t(`settings.nav.${s}`)}
                        </a>
                    ))}
                </aside>

                <div>
                    <Panel
                        id="profile"
                        title={t('settings.profile.title')}
                        intro={t('settings.profile.intro')}
                        saveLabel={t('settings.profile.save')}
                    >
                        <div className="field">
                            <label>{t('settings.profile.avatar')}</label>
                            <div className="avatar-row">
                                <Avatar name="Administrator" size="lg" />
                                <div className="avatar-actions">
                                    <a href="#">
                                        {t('settings.profile.upload')}
                                    </a>
                                    <a href="#">
                                        {t('settings.profile.remove')}
                                    </a>
                                    <div className="hint">
                                        {t('settings.profile.hint')}
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div className="field-row">
                            <div className="field">
                                <label htmlFor="display-name">
                                    {t('settings.profile.displayName')}
                                </label>
                                <input
                                    type="text"
                                    id="display-name"
                                    defaultValue="Administrator"
                                />
                            </div>
                            <div className="field">
                                <label htmlFor="location">
                                    {t('settings.profile.location')}
                                </label>
                                <input
                                    type="text"
                                    id="location"
                                    defaultValue="Amsterdam"
                                />
                            </div>
                        </div>
                        <div className="field">
                            <label htmlFor="website">
                                {t('settings.profile.website')}
                            </label>
                            <input
                                type="url"
                                id="website"
                                placeholder="https://"
                            />
                        </div>
                        <div className="field">
                            <label htmlFor="about">
                                {t('settings.profile.about')}
                            </label>
                            <textarea
                                id="about"
                                defaultValue="Lorem ipsum dolor sit amet, consectetur adipiscing elit."
                            />
                        </div>
                    </Panel>

                    <Panel
                        id="account"
                        title={t('settings.account.title')}
                        intro={t('settings.account.intro')}
                        saveLabel={t('settings.account.save')}
                    >
                        <div className="field-row">
                            <div className="field">
                                <label htmlFor="first-name">
                                    {t('settings.account.firstName')}
                                </label>
                                <input
                                    type="text"
                                    id="first-name"
                                    defaultValue="Robert"
                                />
                            </div>
                            <div className="field">
                                <label htmlFor="last-name">
                                    {t('settings.account.lastName')}
                                </label>
                                <input
                                    type="text"
                                    id="last-name"
                                    defaultValue="Monden"
                                />
                            </div>
                        </div>
                        <div className="field">
                            <label htmlFor="username">
                                {t('settings.account.username')}
                            </label>
                            <input
                                type="text"
                                id="username"
                                defaultValue="admin"
                                disabled
                            />
                            <div className="hint">
                                {t('settings.account.usernameHint')}
                            </div>
                        </div>
                        <div className="field">
                            <label htmlFor="email">
                                {t('settings.account.email')}
                            </label>
                            <input
                                type="email"
                                id="email"
                                defaultValue="admin@example.com"
                            />
                        </div>
                    </Panel>

                    <Panel
                        id="password"
                        title={t('settings.password.title')}
                        intro={t('settings.password.intro')}
                        saveLabel={t('settings.password.save')}
                    >
                        <div className="field">
                            <label htmlFor="current-password">
                                {t('settings.password.current')}
                            </label>
                            <input
                                type="password"
                                id="current-password"
                                autoComplete="current-password"
                            />
                        </div>
                        <div className="field-row">
                            <div className="field">
                                <label htmlFor="new-password">
                                    {t('settings.password.new')}
                                </label>
                                <input
                                    type="password"
                                    id="new-password"
                                    autoComplete="new-password"
                                />
                            </div>
                            <div className="field">
                                <label htmlFor="new-password2">
                                    {t('settings.password.confirm')}
                                </label>
                                <input
                                    type="password"
                                    id="new-password2"
                                    autoComplete="new-password"
                                />
                            </div>
                        </div>
                    </Panel>

                    <Panel
                        id="preferences"
                        title={t('settings.preferences.title')}
                        intro={t('settings.preferences.intro')}
                        saveLabel={t('settings.preferences.save')}
                    >
                        <div className="field-row">
                            <div className="field">
                                <label htmlFor="language">
                                    {t('settings.preferences.language')}
                                </label>
                                <select id="language" defaultValue="en-US">
                                    <option value="en-US">English (US)</option>
                                    <option value="de">Deutsch</option>
                                </select>
                            </div>
                            <div className="field">
                                <label htmlFor="timezone">
                                    {t('settings.preferences.timezone')}
                                </label>
                                <select
                                    id="timezone"
                                    defaultValue="Europe/Amsterdam"
                                >
                                    <option>Europe/Amsterdam</option>
                                    <option>UTC</option>
                                    <option>America/New_York</option>
                                </select>
                            </div>
                        </div>
                        <div className="field">
                            <label>
                                {t('settings.preferences.notifications')}
                            </label>
                            <div className="check">
                                <input
                                    type="checkbox"
                                    id="n-reply"
                                    defaultChecked
                                />
                                <label htmlFor="n-reply" className="inline">
                                    {t('settings.preferences.notifyReply')}
                                </label>
                            </div>
                            <div className="check">
                                <input type="checkbox" id="n-quote" />
                                <label htmlFor="n-quote" className="inline">
                                    {t('settings.preferences.notifyQuote')}
                                </label>
                            </div>
                        </div>
                        <div className="field">
                            <label>{t('settings.preferences.privacy')}</label>
                            <div className="check">
                                <input
                                    type="checkbox"
                                    id="p-online"
                                    defaultChecked
                                />
                                <label htmlFor="p-online" className="inline">
                                    {t('settings.preferences.showOnline')}
                                </label>
                            </div>
                        </div>
                    </Panel>
                </div>
            </div>
        </>
    );
}
