import { memo } from 'react';
import { PageHeader } from './PageHeader';
import { t } from '../../hooks/useI18n';
import TouchpadSettings from '../../components/TouchpadSettings';
import type { Hardware } from './shared';

interface Props {
  hw: Hardware;
}

function TouchpadTab({ hw }: Props) {
  return (
    <>
      <PageHeader title={t('touchpad.title')} />
      <TouchpadSettings
        touchpad={hw.touchpad}
        capabilities={hw.hardwareProfile?.capabilities}
        onSensitivityChange={hw.setTouchpadSensitivity}
        onHapticsChange={hw.setTouchpadHaptics}
        onHapticsIntensityChange={hw.setTouchpadHapticsIntensity}
        onGestureScreenshotChange={hw.setTouchpadGestureScreenshot}
        onRepressChange={hw.setTouchpadRepress}
        onEdgeSlideChange={hw.setTouchpadEdgeSlide}
      />
    </>
  );
}

export default memo(TouchpadTab);
