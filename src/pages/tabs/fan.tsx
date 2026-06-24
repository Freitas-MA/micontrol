import { PageHeader } from './PageHeader';
import { t } from '../../hooks/useI18n';
import FanControl from '../../components/FanControl';
import type { Hardware } from './shared';

interface Props {
  hw: Hardware;
}

export default function FanTab({ hw }: Props) {
  return (
    <>
      <PageHeader title={t('fan.title')} />
      <FanControl fan={hw.fan} onModeChange={hw.setFanMode} />
    </>
  );
}
