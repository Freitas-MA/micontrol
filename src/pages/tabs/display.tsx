import { memo } from 'react';
import { PageHeader } from './PageHeader';
import { t } from '../../hooks/useI18n';
import DisplaySettings from '../../components/DisplaySettings';
import type { Hardware } from './shared';

interface Props {
  hw: Hardware;
}

function DisplayTab({ hw }: Props) {
  return (
    <>
      <PageHeader title={t('display.title')} />
      <DisplaySettings
        display={hw.display}
        capabilities={hw.hardwareProfile?.capabilities}
        onBrightnessChange={hw.setBrightness}
        onHdrChange={hw.setHdr}
        onAiBrightnessChange={hw.setAiBrightness}
        onAiBrightnessConfigChange={hw.setAiBrightnessConfig}
        onRefreshRateChange={hw.setRefreshRate}
        onAdaptiveRefreshRateChange={hw.setAdaptiveRefreshRate}
      />
    </>
  );
}

export default memo(DisplayTab);
