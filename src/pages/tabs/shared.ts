import type { useHardware } from '../../hooks/useHardware';
import type { useSettings } from '../../hooks/useSettings';

export type Hardware = ReturnType<typeof useHardware>;
export type AiSettings = ReturnType<typeof useSettings>;

export interface PerfDebugInfo {
  hq_wmi_instance: string | null;
  hq_wmi_works: boolean;
  hq_wmi_test_ret: string;
  vhf_device_path: string | null;
  registry_mode: string;
  overlay_mode: string;
}
