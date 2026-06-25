import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { invoke } from '@tauri-apps/api/core';
import { useSettings, DEFAULT_SETTINGS } from '../hooks/useSettings';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('useSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === 'get_secret') return Promise.resolve(null);
      if (cmd === 'set_secret') return Promise.resolve(undefined);
      if (cmd === 'delete_secret') return Promise.resolve(undefined);
      return Promise.resolve({});
    });
  });

  afterEach(() => {
    localStorage.clear();
  });

  it('loads default settings on mount', async () => {
    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.settings).toEqual(expect.objectContaining(DEFAULT_SETTINGS));
    });

    expect(result.current.settings.openai_api_key).toBe('');
    expect(result.current.settings.openai_base_url).toBe('https://api.openai.com/v1');
    expect(result.current.settings.openai_model).toBe('gpt-4o-mini');
  });

  it('updateKey updates settings state', async () => {
    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.settings.openai_api_key).toBe('');
    });

    act(() => {
      result.current.updateKey('openai_model', 'gpt-4o');
    });

    await waitFor(() => {
      expect(result.current.settings.openai_model).toBe('gpt-4o');
    });
  });

  it('saveSettings persists to localStorage', async () => {
    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.settings).toBeDefined();
    });

    const updated = {
      ...result.current.settings,
      openai_model: 'gpt-4o',
      openai_api_key: 'sk-test-key',
    };

    await act(async () => {
      await result.current.saveSettings(updated);
    });

    const stored = localStorage.getItem('micontrol_settings_v2');
    expect(stored).not.toBeNull();
    const parsed = JSON.parse(stored!);
    expect(parsed.openai_model).toBe('gpt-4o');
    // API key should NOT be persisted in localStorage
    expect(parsed.openai_api_key).toBeUndefined();
  });

  it('setOnboardingCompleted(true) updates the onboardingCompleted field', async () => {
    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.settings.onboardingCompleted).toBe(false);
    });

    act(() => {
      result.current.setOnboardingCompleted(true);
    });

    await waitFor(() => {
      expect(result.current.settings.onboardingCompleted).toBe(true);
    });
  });

  it('isConfigured returns false when API key is empty, true when set', async () => {
    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.settings.openai_api_key).toBe('');
    });

    expect(result.current.isConfigured).toBe(false);

    act(() => {
      result.current.updateKey('openai_api_key', 'sk-test-key');
    });

    await waitFor(() => {
      expect(result.current.isConfigured).toBe(true);
    });
  });
});
