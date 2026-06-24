import { PageHeader } from './PageHeader';
import { t } from '../../hooks/useI18n';
import UpdateManager from '../../components/UpdateManager';
import type { Hardware } from './shared';

interface Props {
  hw: Hardware;
}

export default function UpdatesTab({ hw }: Props) {
  return (
    <>
      <PageHeader title={t('updates.title')} subtitle={t('updates.subtitle')} />
      <UpdateManager
        updateStatus={hw.updateStatus}
        loadingUpdate={hw.loadingUpdate}
        onRefreshUpdate={hw.refreshUpdateStatus}
      />
    </>
  );
}
