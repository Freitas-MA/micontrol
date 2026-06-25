import { memo } from 'react';
import { PageHeader } from './PageHeader';
import { t } from '../../hooks/useI18n';
import AiAnalysis from '../../components/AiAnalysis';
import type { Hardware, AiSettings } from './shared';

interface Props {
  hw: Hardware;
  ai: AiSettings;
  onOpenSettings: () => void;
}

function AiAnalysisTab({ hw, ai, onOpenSettings }: Props) {
  return (
    <>
      <PageHeader title={t('aiAnalysis.title')} subtitle={t('aiAnalysis.subtitle')} />
      <AiAnalysis hw={hw} ai={ai} onOpenSettings={onOpenSettings} />
    </>
  );
}

export default memo(AiAnalysisTab);
