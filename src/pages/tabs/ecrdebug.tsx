import { PageHeader } from './PageHeader';
import EcrDebugPanel from '../../components/EcrDebugPanel';

export default function EcrDebugTab() {
  return (
    <>
      <PageHeader title="EC Debug Panel" />
      <EcrDebugPanel />
    </>
  );
}
