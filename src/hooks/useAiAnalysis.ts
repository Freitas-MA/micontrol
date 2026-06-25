/**
 * useAiAnalysis
 *
 * AI analysis functions: analyzeSystem, analyzeWithLogs, testConnection.
 * Extracted from `src/hooks/useSettings.ts` (S28-004).
 *
 * Takes the current `AppSettings` as a parameter so it always uses the
 * latest API key / model / base URL.
 */

import { invoke } from '@tauri-apps/api/core';
import type { AppSettings, SystemContext, AnalysisLogEntry } from '../types/settings';
import { buildPrompt, buildLogPrompt } from '../lib/aiPromptBuilder';
import { useTelemetryConsent } from './useTelemetryConsent';

export function useAiAnalysis(settings: AppSettings) {
  const { getTelemetryConsent } = useTelemetryConsent();

  /** Sends system context to the configured AI model and returns the analysis text. */
  async function analyzeSystem(ctx: SystemContext): Promise<string> {
    // Check telemetry consent before sending data
    const consent = await getTelemetryConsent();
    if (consent !== 'granted') {
      throw new Error('consent_denied');
    }

    // Build the prompt locally, send it to the backend command
    const systemContext = buildPrompt(ctx);
    const result = await invoke<string>('analyze_system', {
      systemContext,
      baseUrl: settings.openai_base_url,
      model: settings.openai_model || 'gpt-4o-mini',
      aiDailyAnalyses: settings.ai_daily_analyses,
    });
    return result;
  }

  /** Quick connectivity + auth test — sends a minimal prompt via backend. */
  async function testConnection(): Promise<void> {
    // Check telemetry consent before sending any data to the API
    const consent = await getTelemetryConsent();
    if (consent !== 'granted') {
      throw new Error('consent_denied');
    }

    await invoke<string>('test_connection', {
      baseUrl: settings.openai_base_url,
      model: settings.openai_model || 'gpt-4o-mini',
      aiDailyAnalyses: settings.ai_daily_analyses,
    });
  }

  /**
   * Sends a structured log summary to the AI and returns the analysis text.
   * The AI is instructed to respond in the given language code (en/pt/es/fr).
   */
  async function analyzeWithLogs(
    logs: AnalysisLogEntry[],
    hwCtx: SystemContext,
    language: string,
  ): Promise<string> {
    if (logs.length === 0) throw new Error('no_logs');

    // Check telemetry consent before sending data
    const consent = await getTelemetryConsent();
    if (consent !== 'granted') {
      throw new Error('consent_denied');
    }

    const prompt = buildLogPrompt(logs, hwCtx, language);

    const result = await invoke<string>('analyze_system', {
      systemContext: prompt,
      baseUrl: settings.openai_base_url,
      model: settings.openai_model || 'gpt-4o-mini',
      aiDailyAnalyses: settings.ai_daily_analyses,
    });
    return result;
  }

  return { analyzeSystem, analyzeWithLogs, testConnection };
}
