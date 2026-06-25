import { test, expect, type Page } from '@playwright/test';

/**
 * Default app settings with onboarding completed, so the first-run
 * wizard does not interfere with smoke tests.
 */
const DEFAULT_SETTINGS = {
  onboardingCompleted: true,
  openai_api_key: '',
  openai_base_url: 'https://api.openai.com/v1',
  openai_model: 'gpt-4o-mini',
  perf_mode_ac: null,
  perf_mode_dc: null,
  auto_switch_perf: false,
  tray_opacity: 1.0,
  ai_analysis_enabled: false,
  ai_poll_interval_sec: 60,
  ai_daily_analyses: 2,
};

/** Pre-set localStorage before page loads to skip the onboarding wizard. */
async function skipOnboarding(page: Page) {
  await page.addInitScript((settings) => {
    localStorage.setItem('micontrol_settings_v2', JSON.stringify(settings));
  }, DEFAULT_SETTINGS);
}

/**
 * Dismiss the telemetry consent dialog if it appears.
 *
 * In mock mode, the `get_secret` command returns undefined (unhandled by
 * the mock), so consent resolves to null and the dialog shows on first
 * load. We dismiss it with "Deny" so it doesn't block interactions.
 */
async function dismissConsentDialog(page: Page) {
  const denyButton = page.getByRole('button', { name: 'Deny' });
  await denyButton.waitFor({ state: 'visible', timeout: 5_000 }).catch(() => {});
  if (await denyButton.isVisible().catch(() => false)) {
    await denyButton.click();
    await denyButton.waitFor({ state: 'hidden', timeout: 5_000 }).catch(() => {});
  }
}

test.beforeEach(async ({ page }) => {
  await skipOnboarding(page);
});

// ── Smoke tests ──────────────────────────────────────────────────────────────

test.describe('Smoke tests', () => {
  test('app launches and shows the main window', async ({ page }) => {
    await page.goto('/');
    await dismissConsentDialog(page);

    // The sidebar with the MiControl brand should be visible
    await expect(page.locator('.sidebar-logo')).toBeVisible();
    await expect(page.locator('.sidebar-logo')).toContainText('MiControl');

    // The main content area should be present
    await expect(page.locator('.content-area')).toBeVisible();

    // Navigation items should be present (at least 3 tabs)
    const navItems = page.locator('.sidebar-item');
    await expect(navItems.first()).toBeVisible();
    expect(await navItems.count()).toBeGreaterThanOrEqual(3);
  });

  test('tab navigation works (clicking tabs switches content)', async ({ page }) => {
    await page.goto('/');
    await dismissConsentDialog(page);

    // Click the "Battery" tab
    const batteryTab = page.getByRole('button', { name: 'Battery' }).first();
    await batteryTab.click();
    await expect(batteryTab).toHaveClass(/active/);

    // Click the "About" tab
    const aboutTab = page.getByRole('button', { name: 'About' }).first();
    await aboutTab.click();
    await expect(aboutTab).toHaveClass(/active/);

    // The previously active tab should no longer be active
    await expect(batteryTab).not.toHaveClass(/active/);
  });

  test('settings persist after reload', async ({ page }) => {
    await page.goto('/');
    await dismissConsentDialog(page);

    // Click the "About" tab — this persists the active tab to localStorage
    const aboutTab = page.getByRole('button', { name: 'About' }).first();
    await aboutTab.click();
    await expect(aboutTab).toHaveClass(/active/);

    // Verify the active tab was persisted to localStorage
    const storedTab = await page.evaluate(() => localStorage.getItem('micontrol_active_tab'));
    expect(storedTab).toBe('about');

    // Reload the page
    await page.reload();
    await dismissConsentDialog(page);

    // The "About" tab should still be active after reload
    await expect(aboutTab).toHaveClass(/active/);
  });
});
