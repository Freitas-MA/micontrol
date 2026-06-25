/**
 * useTelemetryConsent
 *
 * Manages telemetry consent state in the OS credential store.
 * Extracted from `src/hooks/useSettings.ts` (S28-004).
 */

import { invoke } from '@tauri-apps/api/core';

const TELEMETRY_CONSENT_KEY = 'telemetry_consent';

/** Privacy policy version — must match POLICY_VERSION in src-tauri/src/util/consent_audit.rs */
const POLICY_VERSION = 2;

export type TelemetryConsentValue = 'granted' | 'denied' | null;

export function useTelemetryConsent() {
  /** Load telemetry consent from the OS credential store. */
  async function getTelemetryConsent(): Promise<TelemetryConsentValue> {
    try {
      const result = await invoke<string | null>('get_secret', { key: TELEMETRY_CONSENT_KEY });
      if (result === 'granted') return 'granted';
      if (result === 'denied') return 'denied';
      return null;
    } catch {
      return null;
    }
  }

  /** Store telemetry consent in the OS credential store. */
  async function setTelemetryConsent(value: 'granted' | 'denied'): Promise<void> {
    try {
      const payload = JSON.stringify({
        value,
        date: new Date().toISOString(),
        policyVersion: POLICY_VERSION,
      });
      await invoke('set_secret', { key: TELEMETRY_CONSENT_KEY, value: payload });
    } catch (err) {
      console.error('[settings] Failed to store telemetry consent:', err);
    }
  }

  /** Revoke telemetry consent — deletes the consent secret and disables AI features. */
  async function revokeTelemetryConsent(): Promise<void> {
    try {
      await invoke('delete_secret', { key: TELEMETRY_CONSENT_KEY });
    } catch {
      // Ignore — key may not exist yet
    }
  }

  return {
    getTelemetryConsent,
    setTelemetryConsent,
    revokeTelemetryConsent,
    checkTelemetryConsent: () => getTelemetryConsent().then((c) => c === 'granted'),
  };
}
