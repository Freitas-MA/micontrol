import { memo } from 'react';
import { t } from '../../hooks/useI18n';
import { PageHeader } from './PageHeader';
import AudioControl from '../../components/AudioControl';
import type { Hardware } from './shared';

interface Props {
  hw: Hardware;
}

function AudioTab({ hw }: Props) {
  return (
    <>
      <PageHeader title={t('audio.pageTitle')} />
      <AudioControl
        audioState={hw.audioState}
        loading={hw.loading}
        onVolumeChange={hw.setMasterVolume}
        onMuteToggle={hw.setMasterMute}
      />
    </>
  );
}

export default memo(AudioTab);
