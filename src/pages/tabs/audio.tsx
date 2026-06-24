import { PageHeader } from './PageHeader';
import AudioControl from '../../components/AudioControl';
import type { Hardware } from './shared';

interface Props {
  hw: Hardware;
}

export default function AudioTab({ hw }: Props) {
  return (
    <>
      <PageHeader title="Audio Control" />
      <AudioControl
        audioState={hw.audioState}
        loading={hw.loading}
        onVolumeChange={hw.setMasterVolume}
        onMuteToggle={hw.setMasterMute}
      />
    </>
  );
}
