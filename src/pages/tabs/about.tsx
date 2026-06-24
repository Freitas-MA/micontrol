import { PageHeader } from './PageHeader';
import { t } from '../../hooks/useI18n';

export default function AboutTab() {
  return (
    <>
      <PageHeader title={t('about.title')} />
      <div className="card">
        <div className="grid-2">
          <div>
            <div className="stat-row">
              <span className="stat-label">{t('about.appName')}</span>
              <span className="stat-value">MiControl</span>
            </div>
            <div className="stat-row">
              <span className="stat-label">{t('about.version')}</span>
              <span className="stat-value">0.1.0</span>
            </div>
            <div className="stat-row">
              <span className="stat-label">{t('about.device')}</span>
              <span className="stat-value">Xiaomi Laptop Pro</span>
            </div>
          </div>
        </div>
        <p style={{ marginTop: 16, fontSize: 12, color: 'var(--color-text-muted)' }}>
          {t('about.description')}
        </p>
      </div>
    </>
  );
}
