import { useTranslation } from 'react-i18next';
import { useApiHealth } from '../model/useApiHealth';

export function ApiStatus() {
    const { t } = useTranslation();
    const status = useApiHealth();
    return <p className="api-status">{t(`api.${status}`)}</p>;
}
